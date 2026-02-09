use self_update::backends::github;
use std::process;
use log::{error, info};

pub fn self_update() {
    match github::Update::configure()
        .repo_owner("Ch1nkara")
        .repo_name("GRIB2Sail")
        .bin_name("grib2sail-cli")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
    {
        Ok(updater) => {
            if let Err(e) = updater.update() {
                error!("Update failed: {}", e);
                process::exit(1);
            } else {
                info!(""); // add a feedline after update() output in stdout
                info!("Updated successfully!");
            }
        }
        Err(e) => {
            error!("Failed to configure updater: {}", e);
            process::exit(1);
        }
    }
}

