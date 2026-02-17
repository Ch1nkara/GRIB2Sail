use clap::ValueEnum;
use strum_macros::Display;
use reqwest::{Client, header::{HeaderMap, InvalidHeaderValue}};
use tokio::sync::mpsc::UnboundedSender;
use thiserror::Error;
use std::io::Error;

#[derive(Clone, Debug)]
pub struct Grib {
    pub model: Model,
    pub step: Step,
    pub days: u32,
    pub latitude_max: f64,
    pub latitude_min: f64,
    pub longitude_max: f64,
    pub longitude_min: f64,
    pub components: Vec<Component>,
    pub content: Vec<u8>,
    pub run: String,
    pub secret: String,
}

#[derive(Clone, ValueEnum, Debug, Display)]
#[clap(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Model {
    Arome,
    AromeAntille,
}

#[derive(Copy, Clone, ValueEnum, Debug, Display)]
#[repr(usize)]
pub enum Step {
    #[clap(name = "1h")]
    #[strum(serialize = "1h")]
    H1 = 1,
    #[clap(name = "3h")]
    #[strum(serialize = "3h")]
    H3 = 3,
    #[clap(name = "6h")]
    #[strum(serialize = "6h")]
    H6 = 6,
    #[clap(name = "12h")]
    #[strum(serialize = "12h")]
    H12 = 12,
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
    FinishedOne,
    FinishedAll
}

#[derive(Debug, Clone)]
pub struct ReqwestData {
    pub client: Client,
    pub events: UnboundedSender<DownloadEvent>,
    pub headers: HeaderMap,
    pub urls: Vec<String>,
}

#[derive(Debug, Error)]
pub enum GribError {
    #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("Invalid Configuration: {0}")]
    InvalidConf(String),

    #[error("IO error: {0}")]
    Io(#[from] Error),

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Error: {0}")]
    Generic(String),
}

