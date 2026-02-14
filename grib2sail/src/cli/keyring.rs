use keyring::Entry;

use grib2sail::{ReqwestData, GribError};

pub fn get_secret(id: &str) -> Result<String, GribError> {
    if let Ok(env_val) = std::env::var(id) {
        return Ok(env_val);
    }
    let entry = Entry::new("grib2sail", id)?;
    let secret = entry.get_password()?;
    Ok(secret)
}

pub fn set_secret(id: &str, secret: &str) -> Result<(), GribError> {
    let entry = Entry::new("grib2sail", id)?;
    entry.set_password(secret)?;
    Ok(())
}

