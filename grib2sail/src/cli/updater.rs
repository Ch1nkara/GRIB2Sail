use grib2sail as g2s;

use log::info;
use self_update::{backends::github, get_target};

pub fn self_update() -> Result<(), g2s::GribError> {
    let target = get_target();
    let bin_name = if cfg!(windows) {
        "grib2sail-cli.exe"
    } else {
        "grib2sail-cli"
    };
    let bin_path = format!("grib2sail-{}/{}", target, bin_name);
    github::Update::configure()
        .repo_owner("Ch1nkara")
        .repo_name("GRIB2Sail")
        .bin_name(bin_name)
        .bin_path_in_archive(&bin_path)
        .target(target)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;
    info!("Updated the cli successfully!");
    Ok(())
}
