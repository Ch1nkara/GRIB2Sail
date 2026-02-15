use log::info;

use crate::core::{ReqwestData, GribError};

pub fn get_token(secret: &str, request: ReqwestData) -> Result<String, GribError> {
    info!("Authenticating to MeteoFrance");
    Ok(secret.to_string())
}

