use keyring::{Entry, Error};
use std::io::stdin;
use log::{warn, info};

use grib2sail::GribError;

pub fn get_secret(id: &str) -> Result<String, GribError> {
    if let Ok(env_val) = std::env::var(id) {
        return Ok(env_val);
    }
    let entry = Entry::new("grib2sail", id)?;
    match entry.get_password() {
        Ok(secret) => return Ok(secret),
        Err(Error::NoEntry) => {
            let mut msg = String::from("For the first use only, you must create and provide");
            msg.push_str(" a free application ID from meteofrance.fr, it will be saved");
            msg.push_str(" locally. See documentation for exact procedure.");
            warn!("{}", msg);

            let mut secret = String::new();
            info!("Enter AROME appId:");
            stdin().read_line(&mut secret)?;

            entry.set_password(&secret)?;
            return Ok(secret)
        }
        Err(e) => return Err(GribError::Keyring(e))
    }
}

pub fn delete_secret(id: &str) -> Result<(), GribError> {
    let entry = Entry::new("grib2sail", id)?;
    entry.delete_credential()?;
    info!("Entry deleted from keyring");
    Ok(())
}

