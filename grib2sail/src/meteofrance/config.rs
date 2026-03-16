use crate::core::{Component, Grib, Model};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub static METEOFRANCE: &[(&str, SocketAddr)] = &[
    (
        "portail-api.meteofrance.fr",
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(137, 129, 43, 99)), 443),
    ),
    (
        "public-api.meteofrance.fr",
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(137, 129, 43, 104)), 443),
    ),
];
pub static TOKEN_URL: &str = "https://portail-api.meteofrance.fr/token";

pub static WIND_V: &str =
    "V_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___";

pub enum UrlType {
    CheckAvailability,
    GribData,
}

pub fn get_urls(grib: &Grib, url_type: UrlType, run: &str) -> Vec<String> {
    let mut url = format!("https://{}/public/", METEOFRANCE[1].0);

    if grib.model.to_string().starts_with("arome") {
        url.push_str("arome/1.0/wcs/MF-NWP-HIGHRES-AROME-");
        match grib.model {
            Model::Arome => url.push_str("001-FRANCE"),
            Model::Arome0025 => url.push_str("0025-FRANCE"),
            Model::AromeAntille => url.push_str("OM-0025-ANTIL"),
            Model::AromeGuyane => url.push_str("OM-0025-GUYANE"),
            Model::AromeIndien => url.push_str("OM-0025-INDIEN"),
            Model::AromeNcaledonie => url.push_str("OM-0025-NCALED"),
            Model::AromePolynesie => url.push_str("OM-0025-POLYN"),
            _ => return Vec::<String>::new(),
        }
    } else if grib.model.to_string().starts_with("arpege") {
        url.push_str("arpege/1.0/wcs/MF-NWP-GLOBAL-ARPEGE-");
        match grib.model {
            Model::Arpege100 => url.push_str("01-ATOURX"),
            Model::Arpege025 => url.push_str("025-GLOBE"),
            _ => return Vec::<String>::new(),
        }
    }
    url.push_str("-WCS/");
    match url_type {
        UrlType::CheckAvailability => {
            url.push_str("GetCapabilities?service=WCS");
            url.push_str("&version=1.3.0");
            url.push_str("&language=eng");
            return vec![url];
        }
        UrlType::GribData => {
            url.push_str("GetCoverage?service=WCS");
            url.push_str("&version=2.0.1");
            url.push_str("&format=application/wmo-grib");
        }
    }
    let mut urls = Vec::new();
    let mut times = Vec::new();
    let mut step = grib.step as usize;
    if step == 1 && grib.model == Model::Arpege100 {
        // Only arpege025 can have a step of 1h, defaulting to 3h
        step = 3;
    }
    if step == 1 && grib.model == Model::Arpege025 && grib.days > 2 {
        // Only the first 2 days can have a step of 1h, the rest will have a 3h step
        for t in (0..=24 * 60 * 60 * 2).step_by(3600) {
            times.push(t);
        }
        for t in (24 * 60 * 60 * 2 + 3 * 3600..=24 * 60 * 60 * grib.days)
            .step_by(3 * 3600)
        {
            times.push(t);
        }
    } else {
        for t in (0..=24 * 60 * 60 * grib.days).step_by(step * 3600) {
            times.push(t);
        }
    }
    for component in &grib.components {
        for time in &times {
            let mut temp_url = url.clone();
            temp_url.push_str(&format!("&subset=time({})", time));
            temp_url.push_str(&format!(
                "&subset=lat({},{})",
                grib.latitude_min, grib.latitude_max,
            ));
            temp_url.push_str(&format!(
                "&subset=long({},{})",
                grib.longitude_min, grib.longitude_max,
            ));
            match component {
                Component::Wind => {
                    let mut windu_url = temp_url;
                    let mut windv_url = windu_url.clone();
                    windu_url.push_str("&coverageid=");
                    windv_url.push_str("&coverageid=");
                    windu_url.push_str("U_COMPONENT_OF_WIND__");
                    windu_url.push_str("SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    windv_url.push_str("V_COMPONENT_OF_WIND__");
                    windv_url.push_str("SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    windu_url.push_str(run);
                    windv_url.push_str(run);
                    windu_url.push_str("&subset=height(10)");
                    windv_url.push_str("&subset=height(10)");
                    urls.push(windu_url);
                    urls.push(windv_url);
                }
                Component::WindGust => {
                    if *time == 0 {
                        continue;
                    }
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("WIND_SPEED_GUST__");
                    temp_url.push_str("SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    temp_url.push_str(run);
                    temp_url.push_str("&subset=height(10)");
                    urls.push(temp_url);
                }
                Component::Pressure => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("PRESSURE__MEAN_SEA_LEVEL___");
                    temp_url.push_str(run);
                    urls.push(temp_url);
                }
                Component::CloudCover => {
                    if *time == 0 {
                        continue;
                    }
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("TOTAL_CLOUD_COVER__");
                    temp_url.push_str("GROUND_OR_WATER_SURFACE___");
                    temp_url.push_str(run);
                    urls.push(temp_url);
                }
            }
        }
    }
    urls
}
