use log::{debug, info};

use crate::utils::config;

pub fn download_grib(mut grib: config::Grib, progress_callback: impl Fn(u8))
-> Result<config::Grib, String> {
    // TODO real stuff
    let total_steps = 100;
    for step in 0..total_steps {
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Calculate progress percentage
        let progress = ((step + 1) * 100 / total_steps) as u8;

        // Notify progress
        progress_callback(progress);
    }
    debug!("example debug log");
    info!("example info log");
    grib.run = Some("19700101_18z".to_string());
    grib.content = Some(b"Faker download from https://example.com".to_vec());
    Ok(grib)
//    Err("Error message".to_string())
}

