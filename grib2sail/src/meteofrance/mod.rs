mod config;
mod token;

use crate::core::{
    DownloadEvent, Grib, GribError, Model, ReqwestData, fetch_data, try_get_url,
};
use crate::iridium::{iridium_connect, iridium_disconnect};
use config::{METEOFRANCE, UrlType, WIND_V, get_urls};
pub use token::get_token;

use chrono::{
    DateTime, Duration as ChronoDuration, SecondsFormat::Secs, TimeZone, Utc,
};
use log::{debug, info, warn};
use regex::Regex;
use reqwest::{
    Client, Method,
    header::{AUTHORIZATION, HeaderValue},
};
use std::{thread::sleep, time::Duration};

pub async fn download_arome_arpege_grib(
    grib: Grib,
    request: ReqwestData,
) -> Result<Grib, GribError> {
    let is_iridium = grib.iridium;
    if is_iridium {
        iridium_connect(METEOFRANCE).await?;
    }
    let grib_res = fetch_arome_arpege_data(grib, request).await;
    if is_iridium {
        iridium_disconnect(METEOFRANCE).await?;
    }
    grib_res
}

async fn fetch_arome_arpege_data(
    mut grib: Grib,
    mut request: ReqwestData,
) -> Result<Grib, GribError> {
    if grib.iridium {
        let mut builder = Client::builder().timeout(Duration::from_secs(60));
        for (host, ip) in METEOFRANCE {
            builder = builder.resolve(host, *ip);
        }
        request.client = builder.build()?;
    }

    let token = get_token(&grib.secret, &request).await?;
    let bearer_header = HeaderValue::from_str(&format!("Bearer {}", token))?;
    request.headers.insert(AUTHORIZATION, bearer_header);

    if grib.days > 2 && grib.model.to_string().starts_with("arome") {
        warn!("Arome forecast is limited to 2 days max");
        grib.days = 2;
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

    if grib.iridium {
        grib.run = guess_last_run()?;
    } else {
        grib.run = find_last_run(&grib, request.clone()).await?;
    }

    request.urls = get_urls(&grib, UrlType::GribData, &grib.run);

    // Change fomatting from 1970-01-01T00.00.00Z to 19700101-00z
    grib.run = grib
        .run
        .replace("-", "")
        .replace("T", "-")
        .chars()
        .take(11)
        .collect::<String>();
    grib.run.push('z');

    info!("Downloading the grib layers");
    let total = request.urls.len();
    let events = request.events.clone();

    events.send(DownloadEvent::Started { total })?;
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
        let chunks = request.urls.chunks(100).collect::<Vec<_>>();
        for (i, chunk) in chunks.iter().enumerate() {
            let mut req = request.clone();
            req.urls = chunk.to_vec();
            grib.content.append(&mut fetch_data(req).await?);
            if i != chunks.len() - 1 {
                info!("Sleeping 1 minute...");
                sleep(Duration::from_mins(1));
            }
        }
    }

    events.send(DownloadEvent::FinishedAll)?;
    Ok(grib)
}

fn guess_last_run() -> Result<String, GribError> {
    let now: DateTime<Utc> = Utc::now();
    let mut last_run = None;
    for day in [-1, 0, 1] {
        for hour in [0, 6, 12, 18] {
            if let Some(dt_naive) = (now.date_naive()
                + ChronoDuration::days(day))
            .and_hms_opt(hour, 0, 0)
            {
                let dt = Utc.from_utc_datetime(&dt_naive);
                if dt <= now - ChronoDuration::hours(5)
                    && dt >= now - ChronoDuration::hours(11)
                {
                    debug!("last_run guessed: {:?}", dt);
                    last_run = Some(dt);
                    break;
                }
            }
        }
        if last_run.is_some() {
            break;
        }
    }
    Ok(last_run
        .ok_or_else(|| {
            GribError::Generic("Failed to guess latest forecast".to_string())
        })?
        .to_rfc3339_opts(Secs, true))
}

async fn find_last_run(
    grib: &Grib,
    mut request: ReqwestData,
) -> Result<String, GribError> {
    info!("Finding the latest available forecast");
    request.urls = get_urls(grib, UrlType::CheckAvailability, "");
    // List the forecasts availables, filter the wind ones and extract
    // the last 2 run dates from the last two lines
    let mut attempts = 0;
    let mut runs_available;
    let mut wind_runs: Vec<&str>;
    let last_two = loop {
        attempts += 1;
        if attempts > 1 {
            info!("Failed to find latest forecast, retrying {}/3", attempts);
            // Backoff before retrying
            sleep(Duration::from_secs(1));
        }
        let req = request
            .client
            .request(Method::GET, &request.urls[0])
            .headers(request.headers.clone())
            .build()?;
        runs_available = match try_get_url(&request.client, req).await {
            Ok(b) => String::from_utf8(b)?,
            Err(e) if attempts < 3 => continue,
            Err(e) => return Err(e),
        };
        wind_runs = runs_available
            .lines()
            .filter(|line| line.contains(WIND_V))
            .collect();
        let last_two = &wind_runs[wind_runs.len().saturating_sub(2)..];
        if last_two.len() == 2 {
            break last_two;
        }
    };

    let last2run = vec![extract_date(last_two[1])?, extract_date(last_two[0])?];
    debug!("last2run: {:?}", last2run);

    let mut run = &last2run[0];
    request.urls = get_urls(grib, UrlType::GribData, run);
    //debug!("Urls generated are: {:?}", layer_urls);

    // If the last run does not have all the required layers yet,
    // fall back to the previous one
    if request
        .client
        .head(&request.urls[request.urls.len() - 1])
        .headers(request.headers.clone())
        .send()
        .await?
        .error_for_status()
        .is_err()
    {
        debug!("missing layer, falling back to previous run");
        run = &last2run[1];
    }
    Ok(run.to_string())
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
