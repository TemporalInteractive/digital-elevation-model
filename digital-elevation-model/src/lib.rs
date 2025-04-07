use std::{ops::Range, path::PathBuf};

use glam::Vec2;
use half::f16;
use image::ImageReader;
use speedy::{Readable, Writable};

pub mod database;

pub use database::*;

/// Digital elevation model profile, describes auxilery data
#[derive(Debug, Clone, Readable, Writable)]
pub struct DemProfile {
    /// Width of the total dem in pixels
    pub width: u32,
    /// Height of the total dem in pixels
    pub height: u32,
    /// Number of meters a single pixel represents inside the dem
    pub meters_per_pixel: f32,
    /// Maximum elevation in meters of the total dem
    pub max_elevation: f32,
}

impl Default for DemProfile {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            meters_per_pixel: 1.0,
            max_elevation: 1.0,
        }
    }
}

/// Digital elevation model
#[derive(Debug, Clone, Readable, Writable)]
pub struct Dem {
    width_range: Range<u32>,
    height_range: Range<u32>,
    profile: DemProfile,
    dem: Vec<u16>,
}

impl Dem {
    pub fn load_chunks_from_image(
        path: &PathBuf,
        chunk_width: u32,
        chunk_height: u32,
        profile: DemProfile,
    ) -> anyhow::Result<Vec<Self>> {
        println!("Loading...");
        let mut image_reader = ImageReader::open(path)?;
        image_reader.no_limits();
        let image = image_reader.decode()?.to_rgb16();
        let width = image.width();
        let height = image.height();
        assert_eq!(width, profile.width);
        assert_eq!(height, profile.height);
        println!("Succesfully loaded {:?}", path);

        let num_chunks_x = width.div_ceil(chunk_width);
        let num_chunks_y = height.div_ceil(chunk_height);

        let mut chunks = Vec::new();
        for y in 0..num_chunks_y {
            for x in 0..num_chunks_x {
                let width_range = (x * chunk_width)..((x + 1) * chunk_width).min(width);
                let height_range = (y * chunk_height)..((y + 1) * chunk_height).min(height);
                let dem = vec![
                    0;
                    ((width_range.end - width_range.start)
                        * (height_range.end - height_range.start))
                        as usize
                ];

                chunks.push(Dem {
                    width_range,
                    height_range,
                    profile: profile.clone(),
                    dem,
                })
            }
        }

        for (x, y, pixel) in image.enumerate_pixels() {
            let (r, _g, _b) = (pixel[0], pixel[1], pixel[2]);

            let elevation = r as f32 / u16::MAX as f32;
            let elevation_u16 = f16::from_f32(elevation).to_bits();

            let chunk_idx_x = x / chunk_width;
            let chunk_idx_y = y / chunk_height;
            let chunk_idx = chunk_idx_y * num_chunks_x + chunk_idx_x;
            let chunk = &mut chunks[chunk_idx as usize];

            let dem_idx_x = x % chunk.width();
            let dem_idx_y = y % chunk.height();
            let dem_idx = dem_idx_y * chunk.width() + dem_idx_x;

            chunk.dem[dem_idx as usize] = elevation_u16;

            if x == width - 1 && y % 100 == 0 {
                println!("Parsing {}/{}", y / 100, height / 100);
            }
        }

        println!("Finished parsing!");

        Ok(chunks)
    }

    /// Width in pixels
    pub fn width(&self) -> u32 {
        self.width_range.end - self.width_range.start
    }

    /// Width offset in pixels of this dem relative to the total dem
    pub fn width_offset(&self) -> u32 {
        self.width_range.start
    }

    /// Height in pixels
    pub fn height(&self) -> u32 {
        self.height_range.end - self.height_range.start
    }

    /// Height offset in pixels of this dem relative to the total dem
    pub fn height_offset(&self) -> u32 {
        self.height_range.start
    }

    /// Get the elevation in meters at pixel (x, y)
    pub fn get_elevation(&self, x: u32, y: u32) -> f32 {
        let elevation_u16 = self.dem[(y * self.width() + x) as usize];
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
        let fx = uv.x * (self.width() - 1) as f32;
        let fy = uv.y * (self.height() - 1) as f32;

        // Integer parts
        let x0 = fx.floor() as u32;
        let y0 = fy.floor() as u32;

        // Ensure we don't read out of bounds by clamping to valid indices
        let x1 = (x0 + 1).min(self.width() - 1);
        let y1 = (y0 + 1).min(self.height() - 1);

        // Fractional parts
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        // Sample the four surrounding texels
        let i = |x: u32, y: u32| -> f32 {
            let elevation_u16 = self.dem[(y * self.width() + x) as usize];
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
