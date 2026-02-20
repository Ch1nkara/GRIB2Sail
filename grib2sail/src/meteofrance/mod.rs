mod config;
mod token;

use crate::core::{DownloadEvent, Grib, GribError, ReqwestData, fetch_data};
use config::{UrlType, get_urls};
use token::get_token;

use log::{debug, info, warn};
use regex::Regex;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use std::{thread::sleep, time::Duration};

pub async fn download_arome_grib(
    mut grib: Grib,
    mut request: ReqwestData,
) -> Result<Grib, GribError> {
    let token = get_token(&grib.secret, &request).await?;
    let bearer_header = HeaderValue::from_str(&format!("Bearer {}", token))?;
    request.headers.insert(AUTHORIZATION, bearer_header);

    if grib.days > 2 {
        warn!("Arome forecast a limited to 2 days max");
        grib.days = 2;
    }

    info!("Finding the latest available forecast");
    request.urls = get_urls(&grib, UrlType::GetCapabilities, "");
    // List the forecast available, filter the wind ones and extract
    // the date from the last two lines
    let forecast_runs_available = request
        .client
        .get(&request.urls[0])
        .headers(request.headers.clone())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let wind_runs: Vec<&str> = forecast_runs_available
        .lines()
        .filter(|line| line.contains(config::WIND_V))
        .collect();
    let last_two = &wind_runs[wind_runs.len().saturating_sub(2)..];

    let last2run = vec![extract_date(last_two[1])?, extract_date(last_two[0])?];
    debug!("last2run: {:?}", last2run);

    let mut run = &last2run[0];
    request.urls = get_urls(&grib, UrlType::GetCoverage, run);
    //debug!("Urls generated are: {:?}", layer_urls);

    // If the last run does not have all the required layers yet,
    // fall back to the previous one
    if request
        .client
        .get(&request.urls[request.urls.len() - 1])
        .headers(request.headers.clone())
        .send()
        .await?
        .error_for_status()
        .is_err()
    {
        run = &last2run[1];
        request.urls = get_urls(&grib, UrlType::GetCoverage, run);
    }

    info!("Downloading the grib layers");
    let total = request.urls.len();
    let events = request.events.clone();

    grib.run = run.to_string();

    let _ = events.send(DownloadEvent::Started { total });
    if total < 100 {
        grib.content = fetch_data(request).await?;
    } else {
        let mut msg = String::from("The requested grib has ");
        msg.push_str(&total.to_string());
        msg.push_str(" layers, but MeteoFrance servers limit requests to 100");
        msg.push_str(" per minutes. This program will sleep 1 minute every");
        msg.push_str(" 100 layers until the complete grib file is downloaded.");
        msg.push_str(" You might want to consider reducing the number of");
        msg.push_str(" layers by increasing the step or reducing the number");
        msg.push_str(" of components");
        warn!("{}", msg);
        for chunk in request.urls.chunks(100) {
            let mut req = request.clone();
            req.urls = chunk.to_vec();
            grib.content.append(&mut fetch_data(req).await?);
            sleep(Duration::from_mins(1));
        }
    }

    let _ = events.send(DownloadEvent::FinishedAll);
    Ok(grib)
}

fn extract_date(line: &str) -> Result<String, GribError> {
    let re = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}\.\d{2}\.\d{2}Z")?;
    match re.find(line) {
        Some(m) => Ok(m.as_str().to_string()),
        None => {
            let mut msg = String::from("Couldn't find latest forecast");
            msg.push_str(&format!(" available from: {}", line));
            Err(GribError::Generic(msg))
        }
    }
}
