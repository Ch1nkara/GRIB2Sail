use super::config;
use crate::core::{ReqwestData, GribError};

use log::{info, debug, error};
use serde::Deserialize;
use serde_json::json;
use reqwest::header::{AUTHORIZATION, HeaderValue};

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
    expires_in: usize,
}

pub async fn get_token(secret: String, request: ReqwestData) -> Result<String, GribError> {
    info!("Authenticating to MeteoFrance");
    debug!("Using secret {}", secret);

    let url_token = config::TOKEN_URL;
    let body = json!({"grant_type": "client_credentials"});
    let header = match HeaderValue::from_str(&format!("Basic {}", secret)) {
        Ok(head) => head,
        Err(e) => {
            let mut msg = String::from("The arome AppId provided is not a valid Bearer");
            msg.push_str(&format!(" token, please provide a correct one: {}", e));
            return Err(GribError::InvalidConf(msg));
        }
    };

    let mut response = request.client
        .post(url_token)
        .header(AUTHORIZATION, header)
        .json(&body)
        .send()
        .await?
        .error_for_status();

    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            let mut msg = String::from("The arome AppId provided has been rejected by");
            msg.push_str(&format!(" meteofrance.fr, please provide a correct one: {}", e));
            return Err(GribError::InvalidConf(msg))
        }
    };

    let token: TokenResponse =response.json().await?;

    debug!("got token: {:?}", token);
    Ok(token.access_token)
}

