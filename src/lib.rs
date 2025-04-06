use std::path::Path;

use glam::Vec2;
use half::f16;
use image::ImageReader;
use speedy::{Readable, Writable};

pub mod database;

/// Digital elevation model profile, describes auxilery data
#[derive(Debug, Clone, Readable, Writable)]
pub struct DemProfile {
    /// Number of meters a single pixel represents inside the dem
    pub meters_per_pixel: f32,
    /// Maximum elevation in meters of the total dem
    pub max_elevation: f32,
}

impl Default for DemProfile {
    fn default() -> Self {
        Self {
            meters_per_pixel: 1.0,
            max_elevation: 10.0,
        }
    }
}

/// Digital elevation model
#[derive(Debug, Clone, Readable, Writable)]
pub struct Dem {
    width: u32,
    height: u32,
    profile: DemProfile,
    dem: Vec<u16>,
}

impl Dem {
    pub fn load_from_image(path: &Path, profile: DemProfile) -> anyhow::Result<Self> {
        let image = ImageReader::open(path)?.decode()?.to_rgb8();

        let width = image.width();
        let height = image.height();

        let mut dem = vec![0; (width * height) as usize];
        for (x, y, pixel) in image.enumerate_pixels() {
            let (r, _g, _b) = (pixel[0], pixel[1], pixel[2]);

            let elevation = r as f32 / 255.0;
            dem[(y * width + x) as usize] = f16::from_f32(elevation).to_bits();
        }

        Ok(Self {
            width,
            height,
            profile,
            dem,
        })
    }

    /// Width in pixels
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the elevation in meters at pixel (x, y)
    pub fn get_elevation(&self, x: u32, y: u32) -> f32 {
        let elevation_u16 = self.dem[(y * self.width + x) as usize];
        let elevation_f16 = f16::from_bits(elevation_u16);
        elevation_f16.to_f32() * self.profile.max_elevation
    }

    /// Sample elevation bilinearly at (latitude, longitude)
    pub fn sample_elevation(&self, latitude: f32, longitude: f32) -> f32 {
        let lon_degrees = longitude.to_degrees();
        let lat_degrees = latitude.to_degrees();
        let uv = Vec2::new((lon_degrees + 180.0) / 360.0, (90.0 - lat_degrees) / 180.0);

        self.sample_elevation_uv(uv)
    }

    /// Sample elevation bilinearly at uv
    pub fn sample_elevation_uv(&self, uv: Vec2) -> f32 {
        // Scale UV to DEM pixel coordinates
        let fx = uv.x * (self.width - 1) as f32;
        let fy = uv.y * (self.height - 1) as f32;

        // Integer parts
        let x0 = fx.floor() as u32;
        let y0 = fy.floor() as u32;

        // Ensure we don't read out of bounds by clamping to valid indices
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        // Fractional parts
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        // Sample the four surrounding texels
        let i = |x: u32, y: u32| -> f32 {
            let elevation_u16 = self.dem[(y * self.width + x) as usize];
            let elevation_f16 = f16::from_bits(elevation_u16);
            elevation_f16.to_f32()
        };

        let v00 = i(x0, y0); // top-left
        let v10 = i(x1, y0); // top-right
        let v01 = i(x0, y1); // bottom-left
        let v11 = i(x1, y1); // bottom-right

        // Bilinear interpolation
        let top = v00 * (1.0 - tx) + v10 * tx;
        let bottom = v01 * (1.0 - tx) + v11 * tx;
        let bilinear = top * (1.0 - ty) + bottom * ty;

        bilinear * self.profile.max_elevation
    }
}
