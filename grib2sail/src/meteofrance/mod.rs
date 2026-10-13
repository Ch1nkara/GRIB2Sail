mod config;
mod token;

use crate::core::{
    DownloadContext, DownloadEvent, Grib, GribError, Model, fetch_data,
    fetch_url_5_try,
};
use config::{UrlType, WIND_V, get_urls};
use token::get_token;
pub use token::is_secret_valid;

use log::{debug, info, warn};
use regex::Regex;
use reqwest::{
    Method,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use std::{thread::sleep, time::Duration};

pub async fn download_grib(
    mut grib: Grib,
    mut ctx: DownloadContext,
) -> Result<(Vec<u8>, String), GribError> {
    let token = get_token(ctx.auth.clone()).await?;
    let bearer_header = HeaderValue::from_str(&format!("Bearer {}", token))?;
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, bearer_header);

    if grib.days > 2 && grib.model.to_string().starts_with("arome") {
        warn!("Arome forecast is limited to 2 days max");
    } else if grib.days > 4 && grib.model.to_string().starts_with("arpege") {
        warn!("Arpege forecast is limited to 4 days max");
        grib.days = 4;
    }
    if grib.step as usize == 1 && grib.model == Model::Arpege100 {
        warn!(
            "Only {} can have a step of 1h, defaulting to 3h",
            Model::Arpege025
        );
    }
    if grib.step as usize == 1
        && grib.model == Model::Arpege025
        && grib.days > 2
    {
        let mut msg = String::from("Only the first 2 days can have a");
        msg.push_str(" step of 1h, the rest will have a 3h step");
        warn!("{}", msg);
    }

    info!("Finding the latest available forecast");
    ctx.urls_headers = get_urls(&grib, UrlType::CheckAvailability, "");
    // List the forecasts availables, filter the wind ones and extract
    // the last 2 run dates from the last two lines
    let mut attempts = 1;
    let mut runs_available;
    let mut wind_runs: Vec<&str>;
    let last_two = loop {
        if attempts > 3 {
            return Err("Unable to find the latest availables forecasts".into());
        }
        let req = ctx
            .client
            .request(Method::GET, &ctx.urls_headers[0].0)
            .headers(headers.clone())
            .build()?;
        runs_available =
            String::from_utf8(fetch_url_5_try(&ctx.client, req).await?)?;
        wind_runs = runs_available
            .lines()
            .filter(|line| line.contains(WIND_V))
            .collect();
        let last_two = &wind_runs[wind_runs.len().saturating_sub(2)..];
        if last_two.len() == 2 {
            break last_two;
        }
        attempts += 1;
    };

    let last2run = vec![extract_date(last_two[1])?, extract_date(last_two[0])?];
    debug!("last2run: {:?}", last2run);

    let mut run = &last2run[0];
    ctx.urls_headers = get_urls(&grib, UrlType::GribData, run);
    //debug!("Urls generated are: {:?}", layer_urls);

    // If the last run does not have all the required layers yet,
    // fall back to the previous one
    if ctx
        .client
        .get(&ctx.urls_headers[ctx.urls_headers.len() - 1].0)
        .headers(headers.clone())
        .send()
        .await?
        .error_for_status()
        .is_err()
    {
        run = &last2run[1];
        ctx.urls_headers = get_urls(&grib, UrlType::GribData, run);
    }

    info!("Downloading the grib layers");
    let total = ctx.urls_headers.len();
    let events = ctx.events.clone();

    // Add token header to every request url
    for (_, h) in ctx.urls_headers.iter_mut() {
        *h = headers.clone();
    }

    // Change fomatting from 1970-01-01T00.00.00Z to 19700101-00z
    let mut run: String = run
        .replace("-", "")
        .replace("T", "-")
        .chars()
        .take(11)
        .collect();
    run.push('z');

    let mut content = Vec::new();
    events.send(DownloadEvent::Started { total })?;
    if total < 100 {
        content = fetch_data(ctx).await?;
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
        let chunks = ctx.urls_headers.chunks(100).collect::<Vec<_>>();
        for (i, chunk) in chunks.iter().enumerate() {
            let mut req = ctx.clone();
            req.urls_headers = chunk.to_vec();
            content.append(&mut fetch_data(req).await?);
            if i != chunks.len() - 1 {
                info!("Sleeping 1 minute...");
                sleep(Duration::from_mins(1));
            }
        }
    }

    events.send(DownloadEvent::FinishedAll)?;
    Ok((content, run))
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
