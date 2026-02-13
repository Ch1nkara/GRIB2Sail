use log::{debug, info};
use reqwest::header::{HeaderValue, CONTENT_TYPE};

use crate::core::{fetch_data, Grib, GribError, DownloadEvent, ReqwestData};

use super::token::get_token;

pub async fn download_grib(mut grib: Grib, mut request: ReqwestData)
-> Result<Grib, GribError> {
    let token = get_token()?;
    let dummy_urls: Vec<String> = vec![
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
    ];

    request.headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    request.urls = dummy_urls;

    let total = request.urls.len();

    let _ = request.events.send(DownloadEvent::Started {total});
    grib.content = fetch_data(request.clone()).await?;

    let _ = request.events.send(DownloadEvent::FinishedAll);
    Ok(grib)
}

