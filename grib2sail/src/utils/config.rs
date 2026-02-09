use clap::ValueEnum;

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
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Model {
    Arome,
    AromeAntille,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Step {
    H1,
    H3,
    H6,
    H12,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum Component {
    Wind,
    WindGust,
    Pressure,
    CloudCover
}

