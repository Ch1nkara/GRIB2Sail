mod config;

pub use config::{
    Component, DownloadEvent, Grib, GribError, Model, ReqwestData, Step,
};

use crate::meteofrance;
use crate::noaa;

use futures::{StreamExt, stream};
use reqwest::{Client, header::HeaderMap};
use tokio::sync::mpsc::UnboundedSender;

pub async fn download_grib(
    mut grib: Grib,
    events: UnboundedSender<DownloadEvent>,
) -> Result<Grib, GribError> {
    let client = Client::new();

    let request = ReqwestData {
        client,
        events,
        headers: HeaderMap::new(),
        urls: Vec::new(),
    };

    if grib.model.to_string().starts_with("arome") {
        grib = meteofrance::download_arome_grib(grib, request).await?;
    } else if grib.model == Model::Gfs {
        grib = noaa::download_gfs_grib(grib, request).await?;
    } else {
        let msg = format!("Unexpected model: {}", grib.model);
        return Err(GribError::InvalidConf(msg));
    }

    Ok(grib)
}

pub async fn fetch_data(request: ReqwestData) -> Result<Vec<u8>, GribError> {
    let results = stream::iter(request.urls.clone().into_iter().enumerate())
        .map(|(idx, url)| {
            let request0 = request.clone();
            async move {
                let resp = request0
                    .client
                    .get(&url)
                    .headers(request0.headers)
                    .send()
                    .await?
                    .error_for_status()?;
                let bytes = resp.bytes().await?;

                let _ = request0.events.send(DownloadEvent::FinishedOne);

                Ok::<_, GribError>((idx, bytes.to_vec()))
            }
        })
        .buffer_unordered(5) // allow up to 5 parallel downloads
        .collect::<Vec<_>>()
        .await;

    // Turn Vec<Result<(idx, data)>> into Result<Vec<(idx, data)>>
    let mut parts: Vec<(usize, Vec<u8>)> =
        results.into_iter().collect::<Result<_, _>>()?;

    // Restore original order
    parts.sort_by_key(|(idx, _)| *idx);

    // Concatenate
    let mut ordered = Vec::new();
    for (_, mut data) in parts {
        ordered.append(&mut data)
    }

    Ok(ordered)
}
