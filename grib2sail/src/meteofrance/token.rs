use super::config;
use crate::core::GribError;

use log::{debug, info};
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    // Unused json keys
    //scope: String,
    //token_type: String,
    //expires_in: usize,
}

pub async fn get_token(secret: &str) -> Result<String, GribError> {
    info!("Authenticating to MeteoFrance");

    let url_token = config::TOKEN_URL;
    let body = json!({"grant_type": "client_credentials"});
    let client = Client::new();
    let header = match HeaderValue::from_str(&format!("Basic {}", secret)) {
        Ok(head) => head,
        Err(e) => {
            let mut msg = String::from("The arome AppId provided is not a");
            msg.push_str(" valid Bearer token, please provide a correct");
            msg.push_str(&format!(" one: {}", e));
            return Err(GribError::InvalidConf(msg));
        }
    };

    let response = client
        .post(url_token)
        .header(AUTHORIZATION, header)
        .json(&body)
        .send()
        .await?
        .error_for_status();

    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            let mut msg = String::from("The arome AppId provided has been");
            msg.push_str(" rejected by meteofrance.fr, please provide a");
            msg.push_str(&format!(" correct one: {}", e));
            return Err(GribError::InvalidConf(msg));
        }
    };

    let token: TokenResponse = response.json().await?;

    debug!("Got token: {:?}", token);
    Ok(token.access_token)
}

pub async fn is_secret_valid(secret: &str) -> bool {
    get_token(secret).await.is_ok()
}
