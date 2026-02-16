use crate::core::{Grib, Model, Component};

pub static AROME_ID: &str = "G2S_AROME_BEARER";
pub static TOKEN_URL: &str = "https://portail-api.meteofrance.fr/token";

pub struct UrlParams {
    pub grib: Grib,
    pub url_type: UrlType,
    pub run: String,
}

pub enum UrlType {
    GetCoverage,
    GetCapabilities,
}

pub fn get_urls(url_params: UrlParams) -> Vec<String> {
    let mut url = String::from("https://public-api.meteofrance.fr/public/arome/1.0/wcs/");
    url.push_str("MF-NWP-HIGHRES-AROME-");
    match url_params.grib.model {
        Model::Arome => url.push_str("0025-FRANCE"),
        Model::AromeAntille => url.push_str("OM-0025-ANTIL"),
    }
    url.push_str("-WCS/");
    match url_params.url_type {
        UrlType::GetCapabilities => {
            url.push_str("GetCapabilities?service=WCS");
            url.push_str("&version=1.3.0");
            url.push_str("&language=eng");
            return vec![url]
        }
        UrlType::GetCoverage => {
            url.push_str("GetCoverage?service=WCS");
            url.push_str("&version=2.0.1");
            url.push_str("&format=application/wmo-grib");
        }
    }
    let mut res = Vec::new();
    for component in &url_params.grib.components {
        for time in (0..=(24*60*60*url_params.grib.days)).step_by(3_600*(url_params.grib.step as usize)) {
            let mut temp_url = url.clone();
            temp_url.push_str(&format!("&subset=time({})", time));
            temp_url.push_str(
                &format!(
                    "&subset=lat({},{})",
                    url_params.grib.latitude_min,
                    url_params.grib.latitude_max,
                )
            );
            temp_url.push_str(
                &format!(
                    "&subset=long({},{})",
                    url_params.grib.longitude_min,
                    url_params.grib.longitude_max,
                )
            );
            match component {
                Component::Wind => {
                    let mut windu_url = temp_url.clone();
                    let mut windv_url = temp_url.clone();
                    windu_url.push_str("&coverageid=");
                    windv_url.push_str("&coverageid=");
                    windu_url.push_str("U_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    windv_url.push_str("V_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    windu_url.push_str(&url_params.run);
                    windv_url.push_str(&url_params.run);
                    windu_url.push_str("&subset=height(10)");
                    windv_url.push_str("&subset=height(10)");
                    res.push(windu_url);
                    res.push(windv_url);
                }
                Component::WindGust => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("WIND_SPEED_GUST__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    temp_url.push_str(&url_params.run);
                    temp_url.push_str("&subset=height(10)");
                    res.push(temp_url);
                }
                Component::Pressure => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("PRESSURE__MEAN_SEA_LEVEL___");
                    temp_url.push_str(&url_params.run);
                    res.push(temp_url);
                }
                Component::CloudCover => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("TOTAL_CLOUD_COVER__GROUND_OR_WATER_SURFACE___");
                    temp_url.push_str(&url_params.run);
                    res.push(temp_url);
                }
            }
        }
    }
    res
}
