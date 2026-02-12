use futures::{stream, StreamExt};
use reqwest::{Client, header::HeaderMap};
use tokio::sync::mpsc::UnboundedSender;

use crate::utils::config::{Grib, ReqwestData, DownloadEvent, GribError};
use crate::meteofrance;


pub async fn download_grib(mut grib: Grib, events: UnboundedSender<DownloadEvent>)
-> Result<Grib, GribError> {
    let client = Client::new();

    let mut request = ReqwestData {
        client: client,
        events: events,
        headers: HeaderMap::new(),
        urls: Vec::new(),
    };

    if grib.model.to_string().starts_with("arome") {
         grib = meteofrance::download_grib(grib, request).await?;
    } else {
        return Err(GribError::InvalidConf(format!("Unexpected model: {}", grib.model)));
    }

    Ok(grib)
}

pub async fn fetch_data(request: ReqwestData)
-> Result<Vec<u8>, GribError> {
    let results = stream::iter(request.urls.clone().into_iter().enumerate())
        .map(|(idx, url)| {
            let client = &request.client;
            let events = request.events.clone();
            let headers = request.headers.clone();
            async move {
                let resp = client
                    .get(&url)
                    .headers(headers)
                    .send()
                    .await?
                    .error_for_status()?;
                let bytes = resp.bytes().await?;

                let _ = events.send(DownloadEvent::FinishedOne);

                Ok::<_, GribError>((idx, bytes.to_vec()))
            }
        })
        .buffer_unordered(8)
        .collect::<Vec<_>>()
       .await;


    // Turn Vec<Result<(idx, data)>> into Result<Vec<(idx, data)>>
    let mut parts: Vec<(usize, Vec<u8>)> = results.into_iter().collect::<Result<_, _>>()?;

    // Restore original order
    parts.sort_by_key(|(idx, _)| *idx);

    // Concatenate
    let mut ordered = Vec::new();
    for (_, mut data) in parts { ordered.append(&mut data) }

    Ok(ordered)
}

