mod config;
mod helper_sect3;
mod helper_sect4;
mod helper_sect5;
mod helper_sect7;
mod unpack_crop;

use config::{UrlType, get_headers, get_urls};
use unpack_crop::unpack_crop;

use crate::core::{Component, DownloadOptions, Grib, GribError, fetch_data};

use log::{debug, info, warn};
use reqwest::Client;

pub async fn download_grib(
    mut grib: Grib,
    client: &Client,
    options: &DownloadOptions,
) -> Result<(Vec<u8>, String), GribError> {
    if grib.days > 15 {
        warn!("ECMWF forecast is limited to 15 days max");
        grib.days = 15;
    }

    info!("Finding the latest available forecast");
    let poss_urls = get_urls(&grib, UrlType::CheckAvailability, "");
    let mut index = 0;
    let run;
    loop {
        if index >= poss_urls.len() {
            return Err("Couldn't find latest available forecast".into());
        }
        let url = &poss_urls[index].0;
        let resp = client.head(url).send().await?.error_for_status();

        if resp.is_ok() {
            run = format!("{}-{}z", &url[54..62], &url[63..65]);
            debug!("found run: {}", run);
            break;
        }
        index += 1;
    }

    info!("Fetching metadata");
    let mut to_download = get_urls(&grib, UrlType::GribData, &run);
    // get the range bytes header for all requested layers
    for (url, header) in to_download.iter_mut() {
        let index = client
            .get(url.replace(".grib2", ".index"))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        *header = get_headers(index, &grib.components)?;
    }

    info!("Downloading the grib layers");
    let nb_range = to_download
        .first()
        .ok_or("Unexpected empty urls_and_headers")?
        .1
        .get("Range")
        .ok_or("Unexpected missing RANGE header")?
        .to_str()?
        .chars()
        .filter(|&c| c == ',')
        .count()
        + 1;

    let total = to_download.len() * nb_range;

    options.started(total)?;

    let content = fetch_data(client, &to_download, options).await?;

    options.finished_all(total)?;

    info!("Unpacking and cropping the grib file");
    let mut nb_components = grib.components.len();
    if grib.components.iter().any(|c| matches!(c, Component::Wind)) {
        nb_components += 1;
    }

    let total = (nb_components * grib.days as usize * 24 / grib.step as usize)
        + nb_components;
    options.started(total)?;

    let content = unpack_crop(&grib, content, options).await?;

    options.finished_all(total)?;

    Ok((content, run))
}
