use log::{debug, info};

use crate::utils::config;

pub fn download_grib(grib: config::Grib) -> Vec<u8> {
    // TODO real stuff
    debug!("example debug log");
    info!("example info log");
    return b"Faker download from https://example.com".to_vec()
}
