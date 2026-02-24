use grib2sail as g2s;

use log::info;
use self_update::backends::github;

pub fn self_update() -> Result<(), g2s::GribError> {
    github::Update::configure()
        .repo_owner("Ch1nkara")
        .repo_name("GRIB2Sail")
        .bin_name("grib2sail-cli")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    info!("Updated the cli successfully!");
    Ok(())
}
