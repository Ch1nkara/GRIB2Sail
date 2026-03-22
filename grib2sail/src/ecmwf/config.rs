use crate::core::{Component, Grib, GribError};

use chrono::{Duration, Local};
use log::{debug, warn};
use reqwest::header::{HeaderMap, HeaderValue, RANGE};

pub static ECMWF_HOST: &str = "ecmwf-forecasts.s3.eu-central-1.amazonaws.com";

pub enum UrlType {
    CheckAvailability,
    GribData,
}

pub fn get_urls(grib: &Grib, url_type: UrlType) -> Vec<(String, HeaderMap)> {
    let mut urls_headers = Vec::new();

    let url = format!("https://{}/", ECMWF_HOST);

    match url_type {
        UrlType::CheckAvailability => {
            let runs = ["18", "12", "06", "00"];
            let today = Local::now().naive_local();
            let dates =
                [today + Duration::days(1), today, today - Duration::days(1)];
            for &date in &dates {
                let run_d = date.format("%Y%m%d").to_string();
                for &run_h in &runs {
                    let dataset = if run_h == "18" || run_h == "06" {
                        "scda"
                    } else {
                        "oper"
                    };
                    urls_headers.push((
                        format!(
                            "{}{}/{}z/ifs/0p25/{}/{}{}0000-{}h-{}-fc.grib2",
                            url,
                            run_d,
                            run_h,
                            dataset,
                            run_d,
                            run_h,
                            grib.days * 24,
                            dataset,
                        ),
                        HeaderMap::new(),
                    ));
                }
            }
        }
        UrlType::GribData => {
            let run_d = &grib.run[..8];
            let run_h = &grib.run[9..11];
            debug!("date is {} and hour is {}", run_d, run_h);
            let dataset = if run_h == "18" || run_h == "06" {
                "scda"
            } else {
                "oper"
            };
            let mut step = grib.step as usize;
            if step == 1 {
                warn!("ECMWF cannot have a step of 1h, defaulting to 3h");
                step = 3;
            }
            let mut hours = Vec::new();
            if step == 3 && grib.days > 6 {
                let mut msg = String::from("Only the first 6 days can have a");
                msg.push_str(" step of 3h, the rest will have a 6h step");
                warn!("{}", msg);
                for h in (0..=144).step_by(3) {
                    hours.push(h);
                }
                for h in (150..=24 * grib.days).step_by(6) {
                    hours.push(h);
                }
            } else {
                for h in (0..=24 * grib.days).step_by(step) {
                    hours.push(h);
                }
            }
            for h in hours {
                urls_headers.push((
                    format!(
                        "{}{}/{}z/ifs/0p25/{}/{}{}0000-{}h-{}-fc.grib2",
                        url, run_d, run_h, dataset, run_d, run_h, h, dataset,
                    ),
                    HeaderMap::new(),
                ));
            }
        }
    }
    urls_headers
}

pub fn get_headers(
    index: String,
    components: &Vec<Component>,
) -> Result<HeaderMap, GribError> {
    let mut headers = HeaderMap::new();
    let mut aliases = Vec::new();
    for component in components {
        match component {
            Component::Wind => {
                aliases.push("10v".to_string());
                aliases.push("10u".to_string());
            }
            Component::WindGust => aliases.push("10fg".to_string()),
            Component::Pressure => aliases.push("sp".to_string()),
            Component::CloudCover => aliases.push("tcc".to_string()),
        }
    }
    let ranges = &format!("bytes={}", parse_index(aliases, index)?);
    debug!("header ranges: {}", ranges);
    headers.insert(RANGE, HeaderValue::from_str(ranges)?);
    Ok(headers)
}

fn parse_index(keys: Vec<String>, index: String) -> Result<String, GribError> {
    let mut ranges = String::new();
    for line in index.lines() {
        if keys
            .iter()
            .any(|k| line.contains(&format!(r#""param": "{}""#, k)))
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let offset_pos = parts
                .iter()
                .position(|&s| s.contains("_offset"))
                .ok_or("Index file error: Missing _offset in index file")?;
            let offset = parts
                .get(offset_pos + 1)
                .ok_or("Index file error: Missing value after _offset")?
                .chars()
                .rev()
                .skip(1)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
                .parse::<usize>()?;

            let length_pos = parts
                .iter()
                .position(|&s| s.contains("_length"))
                .ok_or("Index file error: Missing _length in index file")?;
            let end = parts
                .get(length_pos + 1)
                .ok_or("Index file error: Missing value after _offset")?
                .chars()
                .rev()
                .skip(1)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
                .parse::<usize>()?
                + offset
                - 1;
            ranges += &format!("{}-{},", offset, end);
        }
    }
    ranges.pop();
    if !ranges.contains('-') {
        return Err("Unexpected index file content".into());
    }
    Ok(ranges)
}
