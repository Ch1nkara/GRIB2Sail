mod utils;

pub mod prelude {
    pub use crate::utils::download_grib;
    pub use crate::utils::config::{MODELS, STEPS, DATAS};
}
