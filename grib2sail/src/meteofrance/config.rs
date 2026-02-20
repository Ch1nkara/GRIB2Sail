use crate::core::{Component, Grib, Model};

pub static TOKEN_URL: &str = "https://portail-api.meteofrance.fr/token";
pub static WIND_V: &str = "V_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___";

pub enum UrlType {
    GetCoverage,
    GetCapabilities,
}

pub fn get_urls(grib: &Grib, url_type: UrlType, run: &str) -> Vec<String> {
    let mut url = String::from("https://public-api.meteofrance.fr/public/arome/1.0/wcs/");
    url.push_str("MF-NWP-HIGHRES-AROME-");
    match grib.model {
        Model::Arome => url.push_str("001-FRANCE"),
        Model::Arome0025 => url.push_str("0025-FRANCE"),
        Model::AromeAntille => url.push_str("OM-0025-ANTIL"),
        Model::AromeGuyane => url.push_str("OM-0025-GUYANE"),
        Model::AromeIndien => url.push_str("OM-0025-INDIEN"),
        Model::AromeNcaledonie => url.push_str("OM-0025-NCALED"),
        Model::AromePolynesie => url.push_str("OM-0025-POLYN"),
    }
    url.push_str("-WCS/");
    match url_type {
        UrlType::GetCapabilities => {
            url.push_str("GetCapabilities?service=WCS");
            url.push_str("&version=1.3.0");
            url.push_str("&language=eng");
            return vec![url];
        }
        UrlType::GetCoverage => {
            url.push_str("GetCoverage?service=WCS");
            url.push_str("&version=2.0.1");
            url.push_str("&format=application/wmo-grib");
        }
    }
    let mut res = Vec::new();
    for component in &grib.components {
        for time in (0..=(24 * 60 * 60 * grib.days)).step_by(3_600 * (grib.step as usize)) {
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
                    windu_url
                        .push_str("U_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    windv_url
                        .push_str("V_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    windu_url.push_str(run);
                    windv_url.push_str(run);
                    windu_url.push_str("&subset=height(10)");
                    windv_url.push_str("&subset=height(10)");
                    res.push(windu_url);
                    res.push(windv_url);
                }
                Component::WindGust => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("WIND_SPEED_GUST__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___");
                    temp_url.push_str(run);
                    temp_url.push_str("&subset=height(10)");
                    res.push(temp_url);
                }
                Component::Pressure => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("PRESSURE__MEAN_SEA_LEVEL___");
                    temp_url.push_str(run);
                    res.push(temp_url);
                }
                Component::CloudCover => {
                    temp_url.push_str("&coverageid=");
                    temp_url.push_str("TOTAL_CLOUD_COVER__GROUND_OR_WATER_SURFACE___");
                    temp_url.push_str(run);
                    res.push(temp_url);
                }
            }
        }
    }
    res
}
