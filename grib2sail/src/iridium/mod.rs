mod config;

use crate::core::GribError;
use config::{UrlType, get_req, parse_response};

use log::{debug, info};
use reqwest::{Client, Request};
use std::net::SocketAddr;
use tokio::time::{Duration, Instant, sleep};

pub async fn iridium_connect(socket: SocketAddr) -> Result<(), GribError> {
    info!("Connecting to internet via iridium");
    if let Err(e) = iridium_try_connect(socket).await {
        let _ = iridium_disconnect(socket).await;
        return Err(e);
    }
    Ok(())
}

pub async fn iridium_disconnect(socket: SocketAddr) -> Result<(), GribError> {
    let client = Client::new();
    info!("Closing iridium connection");
    let req = get_req(&client, UrlType::PerformTask, Some((socket, 0)))?;
    let _ = send_request(&client, req).await?;

    loop {
        let conn_stat = get_status(&client).await?[0];

        if conn_stat == 0 {
            break;
        } else {
            sleep(Duration::from_secs(5)).await;
        }
    }
    Ok(())
}

async fn iridium_try_connect(socket: SocketAddr) -> Result<(), GribError> {
    info!("Checking that signal strength is better than 3/5");
    let client = Client::new();
    loop {
        let sig_stg = get_status(&client).await?[1];

        if sig_stg > 3 {
            break;
        } else {
            info!("Signal strangth is only {}/5, waiting...", sig_stg);
            sleep(Duration::from_secs(5)).await;
        }
    }

    info!("Signal strength ok, attempting connection");
    let req = get_req(&client, UrlType::PerformTask, Some((socket, 1)))?;
    let _ = send_request(&client, req).await?;

    info!("Waiting for connexion to be established");
    let start = Instant::now();
    loop {
        let conn_stat = get_status(&client).await?[0];

        if conn_stat == 4 {
            info!("Connexion step 4/4, internet reached via iridium");
            break;
        } else if conn_stat == 0 {
            let mut msg = String::from("Iridium connection was broken,");
            msg.push_str(" wait a few seconds and try again");
            return Err(GribError::Generic(msg));
        } else {
            info!("Connexion step {}/4, waiting...", conn_stat);
            sleep(Duration::from_secs(5)).await;
        }

        if start.elapsed() > Duration::from_mins(2) {
            let mut msg = String::from("Too much time spent trying to");
            msg.push_str(" establish an internet connexion via iridium,");
            msg.push_str(" giving up");
            return Err(GribError::Generic(msg));
        }
    }
    Ok(())
}

async fn get_status(client: &Client) -> Result<Vec<usize>, GribError> {
    let req = get_req(client, UrlType::GetStatus, None)?;
    let resp = send_request(client, req).await?;
    let parsed = parse_response(resp, UrlType::GetStatus)?;
    let result = vec![parsed[0].parse::<usize>()?, parsed[1].parse::<usize>()?];
    Ok(result)
}

async fn send_request(
    client: &Client,
    req: Request,
) -> Result<String, GribError> {
    let mut attempts = 0;
    Ok(loop {
        attempts += 1;
        if attempts > 1 {
            debug!("{}/3 - Retrying request {:?} ", attempts, req);
            // Backoff before retrying
            sleep(Duration::from_secs(1)).await;
        }
        let req_clone = req.try_clone().ok_or_else(|| {
            GribError::Generic("Request try_clone failed".to_string())
        })?;
        let resp = match client.execute(req_clone).await {
            Ok(r) => r,
            Err(e) if attempts < 3 => continue,
            Err(e) => return Err(GribError::Reqwest(e)),
        };
        let resp = resp.error_for_status()?;
        match resp.text().await {
            Ok(t) => break t,
            Err(e) if attempts < 3 => continue,
            Err(e) => return Err(GribError::Reqwest(e)),
        }
    })
}
