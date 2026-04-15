mod config;

pub use config::{
    Component, DownloadEvent, Grib, GribError, Model, ReqwestData, Step,
};

use crate::ecmwf;
use crate::meteofrance;
use crate::noaa;

use log::debug;
use reqwest::{
    Client, Method, Request,
    header::{HeaderMap, HeaderValue, RANGE},
};
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
        urls_headers: Vec::new(),
    };

    if grib.iridium && !grib.model.iridium_compatible() {
        let msg = String::from("This model is not compatible with iridium");
        return Err(GribError::InvalidConf(msg));
    }

    if grib.model.to_string().starts_with("arome")
        || grib.model.to_string().starts_with("arpege")
    {
        grib = meteofrance::download_arome_arpege_grib(grib, request).await?;
    } else if grib.model.to_string().starts_with("gfs") {
        grib = noaa::download_gfs_grib(grib, request).await?;
    } else if grib.model == Model::Ecmwf {
        grib = ecmwf::download_ecmwf_grib(grib, request).await?;
    } else {
        let msg = format!("Unexpected model: {}", grib.model);
        return Err(GribError::Generic(msg));
    }

    Ok(grib)
}

pub async fn fetch_data(request: ReqwestData) -> Result<Vec<u8>, GribError> {
    let semaphore = Arc::new(Semaphore::new(5)); // limit concurrency to 5
    let mut result = Vec::new();
    let mut tasks = Vec::new();

    for (idx, urls_headers) in request.urls_headers.iter().enumerate() {
        let req = request.clone();
        // split the requests by range if a range header is present
        if let Some(range_header) = urls_headers.1.get("range") {
            let range_str = range_header.to_str()?;
            if !range_str.starts_with("bytes=") {
                return Err("Unexpected non-bytes RANGE header".into());
            }
            let ranges = &range_str[6..];
            let ranges_vec = ranges.split(',');
            let nb_r = ranges_vec.clone().count();
            if nb_r < 1 {
                return Err("Unexpected empty RANGE header".into());
            }
            for (idx_r, byte_range) in ranges_vec.enumerate() {
                let mut req = request.clone();
                let mut headers = HeaderMap::new();
                headers.insert(
                    RANGE,
                    HeaderValue::from_str(&format!("bytes={}", byte_range))?,
                );
                req.urls_headers[idx].1 = headers;
                tasks.push((
                    idx * nb_r + idx_r,
                    tokio::spawn(get_url(idx, req, semaphore.clone())),
                ));
            }
        } else {
            tasks.push((
                idx,
                tokio::spawn(get_url(idx, req, semaphore.clone())),
            ));
        }
    }

    // Collect data as they are downloaded by individual tasks
    for task in tasks {
        result.push((task.0, task.1.await??))
    }

    // Restore original order
    result.sort_by_key(|(idx, _)| *idx);

    // Concatenate Vec<Vec<u8 into Vec<u8
    Ok(result.into_iter().flat_map(|(_, data)| data).collect())
}

async fn get_url(
    idx: usize,
    request: ReqwestData,
    semaphore: Arc<Semaphore>,
) -> Result<Vec<u8>, GribError> {
    let _permit = semaphore.acquire_owned().await?;
    let req = request
        .client
        .request(Method::GET, &request.urls_headers[idx].0)
        .headers(request.urls_headers[idx].1.clone())
        .build()?;
    let bytes = fetch_url_5_try(&request.client, req).await?;
    request.events.send(DownloadEvent::FinishedOne)?;
    Ok(bytes)
}

pub async fn fetch_url_5_try(
    client: &Client,
    req: Request,
) -> Result<Vec<u8>, GribError> {
    let mut attempts = 0;
    let bytes = loop {
        attempts += 1;
        if attempts > 1 {
            debug!("Request {:?} failed retrying {}/5", req, attempts);
            // Backoff before retrying
            sleep(Duration::from_secs(1)).await;
        }
        match try_fetch_url(
            client,
            req.try_clone().expect("Failed to clone a Request"),
        )
        .await
        {
            Ok(b) => break b,
            Err(_) if attempts < 5 => continue,
            Err(e) => return Err(e),
        }
    };
    Ok(bytes)
}

async fn try_fetch_url(
    client: &Client,
    req: Request,
) -> Result<Vec<u8>, GribError> {
    debug!("Sending request {:?}", req);
    let resp = client.execute(req).await?;
    let resp = resp.error_for_status()?;
    Ok(resp.bytes().await?.to_vec())
}
