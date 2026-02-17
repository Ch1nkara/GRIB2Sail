mod token;
mod config;

pub use config::AROME_ID;
use token::get_token;
use crate::core::{fetch_data, Grib, GribError, DownloadEvent, ReqwestData};

use log::{debug, info};
use reqwest::header::{HeaderValue, AUTHORIZATION};
use regex::Regex;

pub async fn download_arome_grib(mut grib: Grib, mut request: ReqwestData)
-> Result<Grib, GribError> {
    let token = get_token(grib.secret.clone(), request.clone()).await?;
    let bearer_header = HeaderValue::from_str(&format!("Bearer {}", token))?;
    request.headers.insert(AUTHORIZATION, bearer_header);

    request.urls = config::get_urls(config::UrlParams {
        grib: grib.clone(),
        url_type: config::UrlType::GetCapabilities,
        run: String::new(),
    });
    let capacity = request.client
        .get(&request.urls[0])
        .headers(request.headers.clone())
        .send()
        .await?
        .error_for_status()?;

    let body = capacity.text().await?;
    let lines: Vec<&str> = body.lines().filter(|line| line.contains(config::WIND_V)).collect();
    let last_two = &lines[lines.len().saturating_sub(2)..];
    let last2run = vec![
        extract_date(last_two[1])?,
        extract_date(last_two[0])?,
    ];
    debug!("last2run: {:?}", last2run);

    let mut run = last2run[0].clone();
    request.urls = config::get_urls(config::UrlParams {
        grib: grib.clone(),
        url_type: config::UrlType::GetCoverage,
        run: run.clone(),
    });
    //debug!("Urls generated are: {:?}", layer_urls);

    // If the last run does not have all the required layers yet,
    // fall back to the previous one
    if request.client.get(&request.urls[request.urls.len() - 1])
        .headers(request.headers.clone())
        .send()
        .await?
        .error_for_status()
        .is_err()
    {
        run = last2run[0].clone();
        request.urls = config::get_urls(config::UrlParams {
            grib: grib.clone(),
            url_type: config::UrlType::GetCoverage,
            run: run.clone(),
        });
    }

    let total = request.urls.len();
    debug!("There are {} layers to download", total);

    let _ = request.events.send(DownloadEvent::Started {total});
    grib.content = fetch_data(request.clone()).await?;
    grib.run = run;

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

