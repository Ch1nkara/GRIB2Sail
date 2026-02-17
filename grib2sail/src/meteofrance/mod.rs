mod token;
mod config;

pub use config::AROME_ID;
use token::get_token;
use crate::core::{fetch_data, Grib, GribError, DownloadEvent, ReqwestData};

use log::{debug, info};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use regex::Regex;

pub async fn download_arome_grib(mut grib: Grib, mut request: ReqwestData)
-> Result<Grib, GribError> {
    let token = get_token(grib.secret.clone(), request.clone()).await?;

    let capa_urls = config::get_urls(config::UrlParams {
        grib: grib.clone(),
        url_type: config::UrlType::GetCapabilities,
        run: String::new(),
    });
    let capacity = request.client
        .get(&capa_urls[0])
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?;

    let body = capacity.text().await?;
    let lines: Vec<&str> = body.lines().filter(|line| line.contains(config::WIND_V)).collect();
    let last_two = &lines[lines.len().saturating_sub(2)..];
    debug!("last_two lines are: {:?}", last_two);
    let last2run = vec![
        extract_date(last_two[1])?,
        extract_date(last_two[0])?,
    ];
    debug!("last2run: {:?}", last2run);

    // TODO continue from here
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

fn extract_date(line: &str) -> Result<String, GribError> {
    debug!("Extracting date from line: {}", line);
    let re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}\.\d{2}\.\d{2}Z").unwrap();
    match re.find(line) {
        Some(m) => Ok(m.as_str().to_string()),
        None => {
            let mut msg = String::from("Couldn't find latest run from: ");
            msg.push_str(line);
            Err(GribError::Generic(msg))
        }
    }
}

