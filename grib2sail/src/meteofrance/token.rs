use log::{info, debug};
use serde::Deserialize;
use serde_json::json;

use crate::core::{ReqwestData, GribError};

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

pub async fn get_token(secret: String, request: ReqwestData) -> Result<String, GribError> {
    info!("Authenticating to MeteoFrance");
    debug!("Using secret {}", secret);

    let body = json!({"grant_type": "client_credentials"});
    let url_token = "http://jsonplaceholder.typicode.com/todos/1";

    let response: reqwest::Response = request.client
        .post(url_token)
        .bearer_auth(secret)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    let token: TokenResponse =response.json().await?;

    Ok(token.token)
}

