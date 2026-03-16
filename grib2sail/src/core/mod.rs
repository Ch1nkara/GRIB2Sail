mod config;

pub use config::{
    Component, DownloadEvent, Grib, GribError, Model, ReqwestData, Step,
};

use crate::meteofrance;
use crate::noaa;

use log::info;
use reqwest::{Client, Method, Request, header::HeaderMap};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Semaphore, mpsc::UnboundedSender},
    time::sleep,
};

pub async fn download_grib(
    mut grib: Grib,
    events: UnboundedSender<DownloadEvent>,
) -> Result<Grib, GribError> {
    let client = Client::new();

    let request = ReqwestData {
        client,
        events,
        headers: HeaderMap::new(),
        urls: Vec::new(),
    };

    if grib.model.to_string().starts_with("arome")
        || grib.model.to_string().starts_with("arpege")
    {
        grib = meteofrance::download_arome_arpege_grib(grib, request).await?;
    } else if grib.model.to_string().starts_with("gfs") {
        grib = noaa::download_gfs_grib(grib, request).await?;
    } else {
        let msg = format!("Unexpected model: {}", grib.model);
        return Err(GribError::InvalidConf(msg));
    }

    Ok(grib)
}

pub async fn fetch_data(request: ReqwestData) -> Result<Vec<u8>, GribError> {
    let semaphore = Arc::new(Semaphore::new(5)); // limit concurrency to 5
    let mut result = vec![];

    for (idx, _) in request.urls.iter().enumerate() {
        let _permit = semaphore.clone().acquire_owned().await?;
        let req = request.clone();
        let handle = tokio::spawn(get_url(idx, req));
        result.push(handle.await??);
    }

    // Restore original order
    result.sort_by_key(|(idx, _)| *idx);

    // Concatenate Vec<Vec<u8 into Vec<u8
    Ok(result.into_iter().flat_map(|(_, data)| data).collect())
}

async fn get_url(
    idx: usize,
    request: ReqwestData,
) -> Result<(usize, Vec<u8>), GribError> {
    let mut attempts = 0;
    let bytes = loop {
        attempts += 1;
        if attempts > 1 {
            info!("Layer {} failed retrying {}/3", idx, attempts);
            // Backoff before retrying
            sleep(Duration::from_secs(1)).await;
        }
        let req = request
            .client
            .request(Method::GET, &request.urls[idx])
            .headers(request.headers.clone())
            .build()?;
        match try_get_url(&request.client, req).await {
            Ok(b) => break b,
            Err(e) if attempts < 3 => continue,
            Err(e) => return Err(e),
        }
    };
    request.events.send(DownloadEvent::FinishedOne)?;
    Ok((idx, bytes))
}

pub async fn try_get_url(
    client: &Client,
    req: Request,
) -> Result<Vec<u8>, GribError> {
    let resp = client.execute(req).await?;
    let resp = resp.error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}
