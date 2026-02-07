use log::{debug, info};
pub fn download_grib(url: &str) -> String {
    // TODO real stuff
    debug!("example debug log");
    info!("example info log");
    format!("Faker download from {}", url)
}
