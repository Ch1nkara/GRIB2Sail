use crate::core::{Component, Grib, Model};

use chrono::{Duration, Local};
use log::warn;
use reqwest::header::HeaderMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub static NOAA_HOST: &str = "nomads.ncep.noaa.gov";
pub static NOAA_SOCKET: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(23, 223, 194, 197)), 443);

pub enum UrlType {
    CheckAvailability,
    GribData,
}

pub fn get_urls(
    grib: &Grib,
    url_type: UrlType,
    run: &str,
) -> Vec<(String, HeaderMap)> {
    let mut urls_headers = Vec::new();

    let domain = format!("https://{}/", NOAA_HOST);
    let (model, format) = match grib.model {
        Model::Gfs025 => ("0p25", "pgrb2"),
        Model::Gfs050 => ("0p50", "pgrb2full"),
        Model::Gfs100 => ("1p00", "pgrb2"),
        _ => ("", ""),
    };

    match url_type {
        UrlType::CheckAvailability => {
            let url = "pub/data/nccf/com/gfs/prod/gfs.";
            let runs = ["18", "12", "06", "00"];
            let today = Local::now().naive_local();
            let dates =
                [today + Duration::days(1), today, today - Duration::days(1)];
            for &date in &dates {
                let date_str = date.format("%Y%m%d").to_string();
                for &hour_run in &runs {
                    let layer = format!(
                        "gfs.t{}z.{}.{}.f{:03}",
                        hour_run,
                        format,
                        model,
                        grib.days * 24,
                    );
                    urls_headers.push((
                        format!(
                            "{}{}{}/{}/atmos/{}",
                            domain, url, date_str, hour_run, layer,
                        ),
                        HeaderMap::new(),
                    ));
                }
            }
        }
        UrlType::GribData => {
            let mut url = format!("{}cgi-bin/filter_gfs_{}.pl", domain, model);
            url.push_str(&format!("?dir=%2Fgfs.{}%2Fatmos", run));

            let mut sub = "&subregion=".to_string();
            sub.push_str(&format!("&leftlon={}", grib.longitude_min));
            sub.push_str(&format!("&rightlon={}", grib.longitude_max));
            sub.push_str(&format!("&bottomlat={}", grib.latitude_min));
            sub.push_str(&format!("&toplat={}", grib.latitude_max));

            let pos = run.find("%2F").map(|idx| idx + 3).unwrap_or(0);
            let hour_run = &run[pos..];

            let mut hours = Vec::new();
            let mut step = grib.step as usize;
            if step == 1 && grib.model != Model::Gfs025 {
                warn!(
                    "Only {} can have a step of 1h, defaulting to 3h",
                    Model::Gfs025
                );
                step = 3;
            }
            if step == 1 && grib.days > 5 {
                let mut msg = String::from("Only the first 5 days can have a");
                msg.push_str(" step of 1h, the rest will have a 3h step");
                warn!("{}", msg);
                for h in 0..=120 {
                    hours.push(h);
                }
                for h in (123..=24 * grib.days).step_by(3) {
                    hours.push(h);
                }
            } else {
                for h in (0..=24 * grib.days).step_by(step) {
                    hours.push(h);
                }
            }
            for hour in hours {
                let mut temp_url = url.clone();
                temp_url.push_str(&format!(
                    "&file=gfs.t{}z.{}.{}.f{:03}",
                    hour_run, format, model, hour,
                ));
                for component in &grib.components {
                    match component {
                        Component::Wind => {
                            temp_url.push_str("&var_UGRD=on");
                            temp_url.push_str("&var_VGRD=on");
                            temp_url.push_str("&lev_10_m_above_ground=on");
                        }
                        Component::WindGust => {
                            temp_url.push_str("&var_GUST=on&lev_surface=on");
                        }
                        Component::Pressure => {
                            temp_url.push_str("&var_PRMSL=on");
                            temp_url.push_str("&lev_mean_sea_level=on");
                        }
                        Component::CloudCover => {
                            temp_url.push_str("&var_TCDC=on");
                            temp_url.push_str("&lev_entire_atmosphere=on");
                        }
                    }
                }
                urls_headers
                    .push((format!("{}{}", temp_url, sub), HeaderMap::new()));
            }
        }
    }
    urls_headers
}
