mod config;

use crate::core::{DownloadEvent, Grib, GribError, ReqwestData, fetch_data};
use config::{UrlType, get_urls};

use log::{debug, info, warn};

pub async fn download_gfs_grib(
    mut grib: Grib,
    mut request: ReqwestData,
) -> Result<Grib, GribError> {
    if grib.days > 16 {
        warn!("GFS forecast a limited to 16 days max");
        grib.days = 16;
    }

    info!("Finding the latest available forecast");
    let mut last_run = String::new();
    let mut date = String::new();
    let mut hour = String::new();
    request.urls = get_urls(&grib, UrlType::CheckAvailability, "");
    // The latest forecast is the first one that does not return an error status
    for url in &request.urls {
        let resp = request
            .client
            .head(url)
            .headers(request.headers.clone())
            .send()
            .await?;

        if resp.error_for_status().is_ok() {
            date = url[60..68].to_string();
            hour = url[69..71].to_string();
            last_run = format!("{}%2F{}", date, hour);
            break;
        }
    }
    debug!("Latest available forecast is {}", last_run);
    if last_run.is_empty() {
        let msg = String::from("Couldn't find latest available forecast");
        return Err(GribError::Generic(msg));
    }

    request.urls = get_urls(&grib, UrlType::GribData, &last_run);

    info!("Downloading the grib layers");
    let total = request.urls.len();
    let events = request.events.clone();

    grib.run = format!("{}-{}z", date, hour);

    let _ = events.send(DownloadEvent::Started { total });
    grib.content = fetch_data(request).await?;

    let _ = events.send(DownloadEvent::FinishedAll);
    Ok(grib)
}
