mod config;

use crate::core::{DownloadEvent, Grib, GribError, ReqwestData, fetch_data};
use crate::iridium::{iridium_connect, iridium_disconnect};
use config::{NOAA_HOST, NOAA_SOCKET, UrlType, get_urls};

use log::{debug, info, warn};
use reqwest::Client;
use std::time::Duration;

pub async fn download_gfs_grib(
    grib: Grib,
    request: ReqwestData,
) -> Result<Grib, GribError> {
    let is_iridium = grib.iridium;
    if is_iridium {
        iridium_connect(NOAA_SOCKET).await?;
    }
    let grib_res = fetch_gfs_data(grib, request).await;
    if is_iridium {
        iridium_disconnect(NOAA_SOCKET).await?;
    }
    grib_res
}

async fn fetch_gfs_data(
    mut grib: Grib,
    mut request: ReqwestData,
) -> Result<Grib, GribError> {
    if grib.days > 16 {
        warn!("GFS forecast is limited to 16 days max");
        grib.days = 16;
    }

    if grib.iridium {
        request.client = Client::builder()
            .resolve(NOAA_HOST, NOAA_SOCKET)
            .timeout(Duration::from_secs(120))
            .build()?;
    }

    info!("Finding the latest available forecast");
    let mut last_run = String::new();
    let mut date = String::new();
    let mut hour = String::new();
    request.urls_headers = get_urls(&grib, UrlType::CheckAvailability, "");
    // The latest forecast is the first one that does not return an error status
    for (url, _) in &request.urls_headers {
        let resp = request.client.head(url).send().await?;

        if resp.error_for_status().is_ok() {
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

    request.urls_headers = get_urls(&grib, UrlType::GribData, &last_run);

    info!("Downloading the grib layers");
    let total = request.urls_headers.len();
    let events = request.events.clone();

    grib.run = format!("{}-{}z", date, hour);

    events.send(DownloadEvent::Started { total })?;

    grib.content = fetch_data(request).await?;

    events.send(DownloadEvent::FinishedAll)?;

    Ok(grib)
}
