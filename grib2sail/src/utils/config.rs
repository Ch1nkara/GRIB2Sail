use clap::ValueEnum;
use strum_macros::Display;
use reqwest::header::HeaderMap;

#[derive(Debug)]
pub struct Grib {
    pub model: Model,
    pub step: Step,
    pub days: u32,
    pub latitude_max: f64,
    pub latitude_min: f64,
    pub longitude_max: f64,
    pub longitude_min: f64,
    pub components: Vec<Component>,
    pub content: Option<Vec<u8>>,
    pub run: Option<String>,
}

#[derive(Clone, ValueEnum, Debug, Display)]
#[clap(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Model {
    Arome,
    AromeAntille,
}

#[derive(Clone, ValueEnum, Debug, Display)]
pub enum Step {
    #[clap(name = "1h")]
    #[strum(serialize = "1h")]
    H1,
    #[clap(name = "3h")]
    #[strum(serialize = "3h")]
    H3,
    #[clap(name = "6h")]
    #[strum(serialize = "6h")]
    H6,
    #[clap(name = "12h")]
    #[strum(serialize = "12h")]
    H12,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Component {
    Wind,
    WindGust,
    Pressure,
    CloudCover
}

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started {
        total: usize,
    },
    FinishedOne {
        index: usize,
        total: usize,
    },
    FinishedAll
}

#[derive(Debug, Clone)]
pub struct Urls {
    pub urls: Vec<String>,
    pub headers: HeaderMap,
}
