use std::path::PathBuf;

use digital_elevation_model::{
    database::{DatabaseEntry, MarsMola},
    Dem,
};
use speedy::Writable;

fn main() -> anyhow::Result<()> {
    let path = PathBuf::from("dem-bakery/assets/Mars_MGS_MOLA_DEM_mosaic_global_463m.tif");

    let dem_chunks = Dem::load_chunks_from_image(&path, 1024 * 8, 1024 * 8, MarsMola::DEM_PROFILE)?;

    let result_dir = path.parent().unwrap().join(path.file_stem().unwrap());
    std::fs::create_dir(&result_dir)?;
    for dem_chunk in dem_chunks {
        let bytes = dem_chunk.write_to_vec().unwrap();

        let filename = path.file_stem().unwrap().to_str().unwrap();
        let result_path = result_dir.join(format!(
            "{}_{}_{}.dem",
            filename,
            dem_chunk.width_offset(),
            dem_chunk.height_offset()
        ));

        std::fs::write(result_path, bytes).unwrap();
    }

    Ok(())
}
