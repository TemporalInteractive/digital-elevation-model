use crate::DemProfile;

pub trait DatabaseEntry {
    /// Original data source
    const ORIGINAL_DATA_SRC: &'static str;
    /// Archived data source maintained by this crate
    const DATA_SRC: &'static str;
    /// Associated profile
    const DEM_PROFILE: DemProfile;
}

pub struct MarsHrscMolaBlend;
impl DatabaseEntry for MarsHrscMolaBlend {
    const ORIGINAL_DATA_SRC: &'static str =
        "https://planetarymaps.usgs.gov/mosaic/Mars/HRSC_MOLA_Blend/Mars_HRSC_MOLA_BlendDEM_Global_200mp_v2.tif";

    const DATA_SRC: &'static str =
        "https://drive.google.com/file/d/1G_x3rypkYM_UoqroRskB8oMpKIKr55S3/view?usp=sharing";

    const DEM_PROFILE: DemProfile = DemProfile {
        meters_per_pixel: 1.0,
        max_elevation: 1.0,
    };
}
