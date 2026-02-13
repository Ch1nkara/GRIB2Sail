mod core;
mod meteofrance;

pub use core::download_grib;
pub use core::config::{Grib, Model, Step, Component, DownloadEvent, GribError};
