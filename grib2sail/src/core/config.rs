use clap::ValueEnum;
use keyring::Error as KeyringError;
use log::SetLoggerError;
use regex::Error as RegError;
use reqwest::{
    Client, Error as ReqError,
    header::{HeaderMap, InvalidHeaderValue},
};
use self_update::errors::Error as SelfUpdateError;
use std::string::FromUtf8Error;
use std::{io::Error as IoError, num::ParseIntError};
use strum_macros::Display;
use thiserror::Error;
use tokio::{
    sync::{
        AcquireError,
        mpsc::{UnboundedSender, error::SendError},
    },
    task::JoinError,
};

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
    pub iridium: bool,
}

#[derive(Clone, ValueEnum, Debug, Display, PartialEq)]
#[clap(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Model {
    #[clap(help = "Res 1km, Between 41°N and 51.5°N / -6°W and 10.5°E")]
    Arome,
    #[clap(help = "Res 2.5km, Between 41°N and 51.5°N / -6°W and 10.5°E")]
    Arome0025,
    #[clap(help = "Between 10.4°N and 22.45°N / -67.8°W and -52.2°W")]
    AromeAntille,
    #[clap(help = "Between 1.05°N and 8.95°N / -56.75°W and -46.3°W")]
    AromeGuyane,
    #[clap(help = "Between -25.9°S and -7.25°S / 32.75°E and 67.6°E")]
    AromeIndien,
    #[clap(help = "Between -26°S and -13.75°S / 158.5°E and 171.5°E")]
    AromeNcaledonie,
    #[clap(help = "Between -25.25°S and -12.6°S / -157.5°W and -144.5°W")]
    AromePolynesie,
    #[clap(help = "Gfs 0,25° - 22km, worldwide")]
    Gfs025,
    #[clap(help = "Gfs 0,50° - 45km, worldwide")]
    Gfs050,
    #[clap(help = "Gfs 1,00° - 90km, worldwide")]
    Gfs100,
}

impl Model {
    pub fn iridium_compatible(&self) -> bool {
        matches!(self, Model::Gfs025 | Model::Gfs050 | Model::Gfs100)
    }
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
#[clap(rename_all = "kebab-case")]
pub enum Component {
    Wind,
    WindGust,
    Pressure,
    CloudCover,
}

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started { total: usize },
    FinishedOne,
    FinishedAll,
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
    Reqwest(#[from] ReqError),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Send error: {0}")]
    SendError(#[from] SendError<DownloadEvent>),

    #[error("Join error: {0}")]
    Join(#[from] JoinError),

    #[error("Acquire error: {0}")]
    Acquire(#[from] AcquireError),

    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("Invalid Configuration: {0}")]
    InvalidConf(String),

    #[error("IO error: {0}")]
    Io(#[from] IoError),

    #[error("Keyring error: {0}")]
    Keyring(#[from] KeyringError),

    #[error("Self-Update error: {0}")]
    SelfUpdate(#[from] SelfUpdateError),

    #[error("Regex error: {0}")]
    Regex(#[from] RegError),

    #[error("Logger error: {0}")]
    SetLoggerError(#[from] SetLoggerError),

    #[error("Parse error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Error: {0}")]
    Generic(String),
}
