mod config;

use crate::core::{
    DownloadOptions, Grib, GribError, fetch_data, fetch_url_5_try,
};
use config::{UrlType, get_urls};

use log::{debug, info, warn};
use reqwest::Client;
use reqwest::Method;

pub async fn download_grib(
    grib: Grib,
    client: &Client,
    options: &DownloadOptions,
) -> Result<(Vec<u8>, String), GribError> {
    fetch_gfs_data(grib, client, options).await
}

async fn fetch_gfs_data(
    mut grib: Grib,
    client: &Client,
    options: &DownloadOptions,
) -> Result<(Vec<u8>, String), GribError> {
    if grib.days > 16 {
        warn!("GFS forecast is limited to 16 days max");
        grib.days = 16;
    }

    info!("Finding the latest available forecast");
    let mut last_run = String::new();
    let mut date = String::new();
    let mut hour = String::new();
    let poss_urls = get_urls(&grib, UrlType::CheckAvailability, "");
    // The latest forecast is the first one that does not return an error status
    for (url, _) in &poss_urls {
        let req = client.request(Method::HEAD, url).build()?;
        if fetch_url_5_try(client, req).await.is_ok() {
            date = url[60..68].to_string();
            hour = url[69..71].to_string();
            last_run = format!("{}%2F{}", date, hour);
            break;
        }
    }
    debug!("Latest available forecast is {}", last_run);
    if last_run.is_empty() {
        return Err("Couldn't find latest available forecast".into());
    }

    let to_download = get_urls(&grib, UrlType::GribData, &last_run);

    info!("Downloading the grib layers");
    let total = to_download.len();

    let run = format!("{}-{}z", date, hour);

    options.started(total)?;

    let content = fetch_data(client, &to_download, options).await?;

    options.finished_all(total)?;

    Ok((content, run))
}
