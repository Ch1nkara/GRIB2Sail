use grib2sail as g2s;

use fs::{copy, remove_file, rename};
use log::{debug, error, info};
use std::path::Path;
use std::{fs, process::Command};

pub fn crop_grib(g: &g2s::Grib, out_dir: &Path) -> Result<(), g2s::GribError> {
    info!("Cropping the grib file");
    let mut msg = String::from("Cropping failed. The grib downloaded");
    msg.push_str(" contain the data for all earth, not just the area");
    msg.push_str(" required. This makes it unreadable by some tools such as");
    msg.push_str(" OpenCPN.");
    let latlonbox = format!(
        "sellonlatbox,{},{},{},{}",
        g.longitude_min, g.longitude_max, g.latitude_min, g.latitude_max
    );
    let ingrib = format!("{}_{}_{}.grib2", g.model, g.run, g.step);
    let ingribcopy = format!("{}_{}_{}_copy.grib2", g.model, g.run, g.step);
    let outgrib = format!("{}_{}_{}_small.grib2", g.model, g.run, g.step);
    let args = [&latlonbox, &ingribcopy, &outgrib];

    #[cfg(target_os = "windows")]
    {
        error!("Cropping function is not yet implemented on Windows CLI");
        Err(g2s::GribError::Generic(msg))
    }

    #[cfg(not(target_os = "windows"))]
    {
        if Command::new("cdo").arg("--help").output().is_err() {
            let mut miss = String::from("Missing dependency, please insall");
            miss.push_str(" cdo and try again. The Climate Data Operator can");
            miss.push_str(" be installed via `apt install cdo` on Ubuntu");
            miss.push_str(" machines");
            error!("{}", miss);
            return Err(g2s::GribError::Generic(msg));
        }
        copy(out_dir.join(&ingrib), &ingribcopy)?;
        debug!("Executing command: {:?}", Command::new("cdo").args(args));
        let output = Command::new("cdo").args(args).output()?;

        if !output.status.success() {
            let mut stderr = String::from("Cropping failed with stderr: ");
            stderr += &String::from_utf8_lossy(&output.stderr);
            error!("{}", stderr);
            return Err(g2s::GribError::Generic(msg));
        }

        remove_file(ingribcopy)?;
        remove_file(out_dir.join(&ingrib))?;
        rename(&outgrib, out_dir.join(ingrib))?;

        Ok(())
    }
}
