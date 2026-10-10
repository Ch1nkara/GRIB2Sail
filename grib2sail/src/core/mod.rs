mod config;

pub use config::{
    Component, DownloadContext, DownloadEvent, Grib, GribError, Model, Step,
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
use tokio::{sync::Semaphore, time::sleep};

pub async fn download_grib(
    grib: Grib,
    ctx: DownloadContext,
) -> Result<(Vec<u8>, String), GribError> {
    if grib.model.to_string().starts_with("arome")
        || grib.model.to_string().starts_with("arpege")
    {
        meteofrance::download_grib(grib, ctx).await
    } else if grib.model.to_string().starts_with("gfs") {
        noaa::download_grib(grib, ctx).await
    } else if grib.model == Model::Ecmwf {
        ecmwf::download_grib(grib, ctx).await
    } else {
        let msg = format!("Unexpected model: {}", grib.model);
        Err(GribError::Generic(msg))
    }
}

pub async fn fetch_data(ctx: DownloadContext) -> Result<Vec<u8>, GribError> {
    let semaphore = Arc::new(Semaphore::new(5)); // limit concurrency to 5
    let mut result = Vec::new();
    let mut tasks = Vec::new();

    for (idx, urls_headers) in ctx.urls_headers.iter().enumerate() {
        let req = ctx.clone();
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
                let mut req = ctx.clone();
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
    ctx: DownloadContext,
    semaphore: Arc<Semaphore>,
) -> Result<Vec<u8>, GribError> {
    let _permit = semaphore.acquire_owned().await?;
    let req = ctx
        .client
        .request(Method::GET, &ctx.urls_headers[idx].0)
        .headers(ctx.urls_headers[idx].1.clone())
        .build()?;
    let bytes = fetch_url_5_try(&ctx.client, req).await?;
    ctx.events.send(DownloadEvent::FinishedOne)?;
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
