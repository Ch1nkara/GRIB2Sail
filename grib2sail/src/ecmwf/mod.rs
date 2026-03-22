mod config;

use config::{UrlType, get_headers, get_urls};

use crate::core::{DownloadEvent, Grib, GribError, ReqwestData, fetch_data};

use log::{debug, info, warn};

pub async fn download_ecmwf_grib(
    mut grib: Grib,
    mut request: ReqwestData,
) -> Result<Grib, GribError> {
    if grib.days > 15 {
        warn!("ECMWF forecast is limited to 15 days max");
        grib.days = 15;
    }

    info!("Finding the latest available forecast");
    request.urls_headers = get_urls(&grib, UrlType::CheckAvailability);
    let mut index = 0;
    loop {
        if index >= request.urls_headers.len() {
            return Err("Couldn't find latest available forecast".into());
        }
        let url = &request.urls_headers[index].0;
        let resp = request.client.head(url).send().await?.error_for_status();

        if resp.is_ok() {
            grib.run = format!("{}-{}z", &url[54..62], &url[63..65]);
            debug!("found run: {}", grib.run);
            break;
        }
        index += 1;
    }

    info!("Fetching metadata");
    request.urls_headers = get_urls(&grib, UrlType::GribData);
    // get the range bytes header for all requested layers
    for (url, h) in request.urls_headers.iter_mut() {
        let index = request
            .client
            .get(url.replace(".grib2", ".index"))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        *h = get_headers(index, &grib.components)?;
    }

    info!("Downloading the grib layers");
    let nb_range = request
        .urls_headers
        .first()
        .ok_or("Unexpected empty urls_headers")?
        .1
        .get("Range")
        .ok_or("Unexpected missing RANGE header")?
        .to_str()?
        .chars()
        .filter(|&c| c == ',')
        .count()
        + 1;

    let total = request.urls_headers.len() * nb_range;

    let events = request.events.clone();

    events.send(DownloadEvent::Started { total })?;

    grib.content = fetch_data(request).await?;

    events.send(DownloadEvent::FinishedAll)?;

    Ok(grib)
}
