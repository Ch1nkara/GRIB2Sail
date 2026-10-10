mod config;
mod helper_sect3;
mod helper_sect4;
mod helper_sect5;
mod helper_sect7;
mod unpack_crop;

use config::{UrlType, get_headers, get_urls};
use unpack_crop::unpack_crop;

use crate::core::{
    Component, DownloadContext, DownloadEvent, Grib, GribError, fetch_data,
};

use log::{debug, info, warn};

pub async fn download_grib(
    mut grib: Grib,
    mut ctx: DownloadContext,
) -> Result<(Vec<u8>, String), GribError> {
    if grib.days > 15 {
        warn!("ECMWF forecast is limited to 15 days max");
        grib.days = 15;
    }

    info!("Finding the latest available forecast");
    ctx.urls_headers = get_urls(&grib, UrlType::CheckAvailability, "");
    let mut index = 0;
    let run;
    loop {
        if index >= ctx.urls_headers.len() {
            return Err("Couldn't find latest available forecast".into());
        }
        let url = &ctx.urls_headers[index].0;
        let resp = ctx.client.head(url).send().await?.error_for_status();

        if resp.is_ok() {
            run = format!("{}-{}z", &url[54..62], &url[63..65]);
            debug!("found run: {}", run);
            break;
        }
        index += 1;
    }

    info!("Fetching metadata");
    ctx.urls_headers = get_urls(&grib, UrlType::GribData, &run);
    // get the range bytes header for all requested layers
    for (url, h) in ctx.urls_headers.iter_mut() {
        let index = ctx
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
    let nb_range = ctx
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

    let total = ctx.urls_headers.len() * nb_range;

    let events = ctx.events.clone();

    events.send(DownloadEvent::Started { total })?;

    let content = fetch_data(ctx).await?;

    events.send(DownloadEvent::FinishedAll)?;

    info!("Unpacking and cropping the grib file");
    let mut nb_components = grib.components.len();
    if grib.components.iter().any(|c| matches!(c, Component::Wind)) {
        nb_components += 1;
    }

    let total = (nb_components * grib.days as usize * 24 / grib.step as usize)
        + nb_components;
    events.send(DownloadEvent::Started { total })?;

    let content = unpack_crop(&grib, content, events.clone()).await?;

    events.send(DownloadEvent::FinishedAll)?;

    Ok((content, run))
}
