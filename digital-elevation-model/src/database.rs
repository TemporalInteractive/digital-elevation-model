use crate::DemProfile;

pub trait DatabaseEntry {
    /// Original data source
    const ORIGINAL_DATA_SRC: &'static str;
    /// Archived data source maintained by this crate
    const ARCHIVED_DATA_SRC: &'static str;
    /// Associated dem profile
    const DEM_PROFILE: DemProfile;
    /// Chunk size in pixels
    const CHUNK_SIZE: u32;
}

pub struct MarsHrscMolaBlend;
impl DatabaseEntry for MarsHrscMolaBlend {
    const ORIGINAL_DATA_SRC: &'static str =
        "https://planetarymaps.usgs.gov/mosaic/Mars/HRSC_MOLA_Blend/Mars_HRSC_MOLA_BlendDEM_Global_200mp_v2.tif";
    const ARCHIVED_DATA_SRC: &'static str =
        "https://drive.google.com/file/d/1G_x3rypkYM_UoqroRskB8oMpKIKr55S3/view?usp=sharing";

    const DEM_PROFILE: DemProfile = DemProfile {
        width: todo!(),
        height: todo!(),
        meters_per_pixel: 200.0,
        max_elevation: 1.0,
    };

    const CHUNK_SIZE: u32 = 1024 * 8;
}

pub struct MarsMola;
impl DatabaseEntry for MarsMola {
    const ORIGINAL_DATA_SRC: &'static str =
        "https://planetarymaps.usgs.gov/mosaic/Mars_MGS_MOLA_DEM_mosaic_global_463m.tif";
    const ARCHIVED_DATA_SRC: &'static str =
        "https://drive.google.com/file/d/1wWgo6Fg_CNKGA26MO2i125k1knwDA6fs/view?usp=sharing";

    const DEM_PROFILE: DemProfile = DemProfile {
        width: 46080,
        height: 23040,
        meters_per_pixel: 463.0,
        max_elevation: 1.0,
    };

    const CHUNK_SIZE: u32 = 1024 * 8;
}
