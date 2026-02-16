mod token;
mod config;

pub use config::AROME_ID;
use token::get_token;
use crate::core::{fetch_data, Grib, GribError, DownloadEvent, ReqwestData};

use log::{debug, info};
use reqwest::header::{HeaderValue, CONTENT_TYPE};

pub async fn download_arome_grib(mut grib: Grib, mut request: ReqwestData)
-> Result<Grib, GribError> {
    //let token = get_token(grib.secret.clone(), request.clone()).await?;

    let dummy_urls: Vec<String> = vec![
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
    ];

    let url_params = config::UrlParams {
        grib: grib.clone(),
        url_type: config::UrlType::GetCoverage,
        run: "FAKERUN".to_string(),
    };
    let dummy_urls = config::get_urls(url_params);
    debug!("Urls generated are: {:?}", dummy_urls);

    request.headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    request.urls = dummy_urls;

    let total = request.urls.len();

    let _ = request.events.send(DownloadEvent::Started {total});
    grib.content = fetch_data(request.clone()).await?;

    let _ = request.events.send(DownloadEvent::FinishedAll);
    Ok(grib)
}

