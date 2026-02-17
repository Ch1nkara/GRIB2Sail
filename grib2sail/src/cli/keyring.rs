use grib2sail as g2s;

use keyring::{Entry, Error};
use log::{error, info, warn};
use std::io::stdin;

static AROME_ID: &str = "G2S_AROME_BEARER";

pub fn get_secret(model: &g2s::Model) -> Result<String, g2s::GribError> {
    if model.to_string().starts_with("arome") {
        match get_password(AROME_ID) {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("{}", e);
                let mut msg = String::from("No password storing solution available, install");
                msg.push_str(" one or use the '");
                msg.push_str(AROME_ID);
                msg.push_str("' environement variable");
                Err(g2s::GribError::Generic(msg))
            }
        }
    } else {
        Ok(String::new())
    }
}

fn get_password(id: &str) -> Result<String, g2s::GribError> {
    if let Ok(env_val) = std::env::var(id) {
        return Ok(env_val);
    }
    let entry = Entry::new("grib2sail", id)?;
    match entry.get_password() {
        Ok(secret) => Ok(secret),
        Err(Error::NoEntry) => {
            let mut msg = String::from("For the first use only, you must create and provide");
            msg.push_str(" a free application ID from meteofrance.fr, it will be saved");
            msg.push_str(" locally. See documentation for exact procedure.");
            warn!("{}", msg);

            let mut secret = String::new();
            info!("Enter AROME appId:");
            stdin().read_line(&mut secret)?;

            let secret = secret.trim_end();
            entry.set_password(secret)?;
            Ok(secret.to_string())
        }
        Err(e) => Err(g2s::GribError::Keyring(e)),
    }
}

pub fn delete_secret(model: &g2s::Model) -> Result<(), g2s::GribError> {
    if model.to_string().starts_with("arome") {
        let entry = Entry::new("grib2sail", AROME_ID)?;
        entry.delete_credential()?;
        info!("Entry deleted from keyring");
    } else {
        warn!("This model has no secret linked, skipping");
    }
    Ok(())
}
