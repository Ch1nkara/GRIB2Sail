use grib2sail as g2s;

use keyring::{Entry, Error};
use log::{error, info, warn};
use reqwest::Client;
use std::io::stdin;
use tokio::sync::mpsc::unbounded_channel;

static ID: &str = "G2S_METEOFRANCE_BEARER";

pub async fn get_secret(model: &g2s::Model) -> Result<String, g2s::GribError> {
    let id = match model.to_string() {
        s if s.starts_with("arome") || s.starts_with("arpege") => ID,
        _ => return Ok(String::new()),
    };
    match get_password(id).await {
        Ok(s) => Ok(s),
        Err(e) => {
            error!("{}", e);
            let mut msg = String::from("No password storing solution");
            msg.push_str(" available, install one or use the '");
            msg.push_str(id);
            msg.push_str("' environement variable");
            Err(g2s::GribError::Generic(msg))
        }
    }
}

async fn get_password(id: &str) -> Result<String, g2s::GribError> {
    if let Ok(env_val) = std::env::var(id) {
        return Ok(env_val);
    }
    let entry = Entry::new("grib2sail", id)?;
    match entry.get_password() {
        Ok(secret) => Ok(secret),
        Err(Error::NoEntry) => {
            let mut msg = String::from("For the first use only, you must");
            msg.push_str(" create and provide a free application ID from");
            msg.push_str(" meteofrance.fr, it will be saved locally.");
            msg.push_str(" See documentation for exact procedure.");
            warn!("{}", msg);

            let (tx, _rx) = unbounded_channel();
            let request = g2s::ReqwestData {
                client: Client::new(),
                events: tx,
                urls_headers: Vec::new(),
            };
            let mut secret = String::new();

            loop {
                let mut msg = String::from("Enter API subscription (free upon");
                msg.push_str(" registration on meteofrance.fr):");
                info!("{}", msg);
                stdin().read_line(&mut secret)?;
                secret = secret.trim_end().to_string();
                info!("Verifying the provided subscription...");
                if g2s::get_token(&secret, &request).await.is_ok() {
                    break;
                }
                error!("The API subbscription is not valid, try again");
            }

            entry.set_password(&secret)?;
            info!("API subscription saved locally");
            Ok(secret.to_string())
        }
        Err(e) => Err(g2s::GribError::Keyring(e)),
    }
}

pub fn delete_secrets() -> Result<(), g2s::GribError> {
    let entry = Entry::new("grib2sail", ID)?;
    entry.delete_credential()?;
    info!("Entry {} deleted from keyring", ID);
    Ok(())
}
