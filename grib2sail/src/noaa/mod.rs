mod config;

use crate::core::{
    DownloadEvent, Grib, GribError, ReqwestData, fetch_data, fetch_url_5_try,
};
use config::{UrlType, get_urls};

use log::{debug, info, warn};
use reqwest::Method;

pub async fn download_gfs_grib(
    grib: Grib,
    request: ReqwestData,
) -> Result<Grib, GribError> {
    let grib_res = fetch_gfs_data(grib, request).await;
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

    info!("Finding the latest available forecast");
    let mut last_run = String::new();
    let mut date = String::new();
    let mut hour = String::new();
    request.urls_headers = get_urls(&grib, UrlType::CheckAvailability, "");
    // The latest forecast is the first one that does not return an error status
    for (url, _) in &request.urls_headers {
        let req = request.client.request(Method::HEAD, url).build()?;
        if fetch_url_5_try(&request.client, req).await.is_ok() {
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
