mod config;

pub(crate) use config::GribError;
pub use config::{Component, DownloadOptions, Grib, Model, Step};

use crate::ecmwf;
use crate::meteofrance;
use crate::noaa;

use log::{debug, info};
use reqwest::{
    Client, Method, Request, StatusCode,
    header::{HeaderMap, HeaderValue, RANGE},
};
use std::{sync::Arc, sync::atomic::{AtomicBool, Ordering}, time::Duration};
use tokio::{sync::Semaphore, time::sleep};

static TOO_MANY_REQUESTS_LOGGED: AtomicBool = AtomicBool::new(false);

pub async fn download_grib(
    grib: Grib,
    options: DownloadOptions,
) -> Result<(Vec<u8>, String), String> {
    download_grib_inner(grib, options)
        .await
        .map_err(|e| e.to_string())
}

async fn download_grib_inner(
    grib: Grib,
    options: DownloadOptions,
) -> Result<(Vec<u8>, String), GribError> {
    let client = Client::new();
    if grib.model.is_meteofrance() {
        meteofrance::download_grib(grib, &client, &options).await
    } else if grib.model.is_gfs() {
        noaa::download_grib(grib, &client, &options).await
    } else if grib.model == Model::Ecmwf {
        ecmwf::download_grib(grib, &client, &options).await
    } else {
        let msg = format!("Unexpected model: {}", grib.model);
        Err(GribError::Generic(msg))
    }
}

pub async fn fetch_data(
    client: &Client,
    urls_and_headers: &[(String, HeaderMap)],
    options: &DownloadOptions,
) -> Result<Vec<u8>, GribError> {
    let semaphore = Arc::new(Semaphore::new(5)); // limit concurrency to 5
    let mut result = Vec::new();
    let mut tasks = Vec::new();

    for (idx, (url, headers_map)) in urls_and_headers.iter().enumerate() {
        // split the requests by range if a range header is present
        if let Some(range_header) = headers_map.get("range") {
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
                let mut headers = HeaderMap::new();
                headers.insert(
                    RANGE,
                    HeaderValue::from_str(&format!("bytes={}", byte_range))?,
                );
                tasks.push((
                    idx * nb_r + idx_r,
                    tokio::spawn(get_url(
                        client.clone(),
                        url.clone(),
                        headers,
                        semaphore.clone(),
                        options.clone(),
                    )),
                ));
            }
        } else {
            tasks.push((
                idx,
                tokio::spawn(get_url(
                    client.clone(),
                    url.to_string(),
                    headers_map.clone(),
                    semaphore.clone(),
                    options.clone(),
                )),
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
    client: Client,
    url: String,
    headers: HeaderMap,
    semaphore: Arc<Semaphore>,
    options: DownloadOptions,
) -> Result<Vec<u8>, GribError> {
    let _permit = semaphore.acquire_owned().await?;
    let req = client.request(Method::GET, &url).headers(headers).build()?;
    let bytes = fetch_url_5_try(&client, req).await?;
    options.finished_one(1)?;
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
        }
        let resp = client.execute(req.try_clone().expect("Failed to clone a Request")).await?;
        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            if attempts < 5 {
                if !TOO_MANY_REQUESTS_LOGGED.swap(true, Ordering::AcqRel) {
                    info!("Too Many Requests, sleeping 1 minute...");
                }
                sleep(Duration::from_secs(60)).await;
                TOO_MANY_REQUESTS_LOGGED.store(false, Ordering::Release);
                continue;
            }
            return Err(GribError::Generic("Too many requests (429)".to_string()));
        }

        match resp.error_for_status() {
            Ok(resp) => break resp.bytes().await?.to_vec(),
            Err(e) if attempts < 5 => {
                sleep(Duration::from_secs(1)).await;
                debug!("Request {:?} failed retrying {}/5: {}", req, attempts, e);
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    };
    Ok(bytes)
}
