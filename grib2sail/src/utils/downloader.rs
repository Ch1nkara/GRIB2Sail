use futures::{stream, StreamExt};
use reqwest::Client;
use tokio::sync::mpsc::UnboundedSender;

use crate::utils::config::{Grib, DownloadEvent};
use crate::meteofrance;

pub async fn download_grib(mut grib: Grib, events: UnboundedSender<DownloadEvent>)
-> Result<Grib, reqwest::Error> {
    let client = Client::new();
    let total = 100;

    let _ = events.send(DownloadEvent::Started {total});

    let urls: Vec<String> = vec![
        String::from("https://jsonplaceholder.typicode.com/todos/1"),
        String::from("https://jsonplaceholder.typicode.com/todos/1"),
        String::from("https://jsonplaceholder.typicode.com/todos/1"),
    ];

    let results = stream::iter(urls.into_iter().enumerate())
        .map(|(idx, url)| {
            let client = &client;
            let events = events.clone();
            async move {
                let resp = client.get(&url).send().await?.error_for_status()?;
                let bytes = resp.bytes().await?;

                let _ = events.send(DownloadEvent::FinishedOne {
                    index: idx,
                    total,
                });

                Ok::<_, reqwest::Error>((idx, bytes.to_vec()))
            }
        })
        .buffer_unordered(8)
        .collect::<Vec<_>>()
       .await;

    let _ = events.send(DownloadEvent::FinishedAll);

    // Turn Vec<Result<(idx, data)>> into Result<Vec<(idx, data)>>
    let mut parts: Vec<(usize, Vec<u8>)> =
        results.into_iter().collect::<Result<_, _>>()?;

    // Restore original order
    parts.sort_by_key(|(idx, _)| *idx);

    // Concatenate
    let mut ordered = Vec::new();
    for (_, mut data) in parts {
        ordered.append(&mut data);
    }

    //grib.content = results.into_iter().map(|v| v.unwrap()).collect();
    grib.content = Some(ordered);

    Ok(grib)
/*    if grib.model.to_string().starts_with("arome") {
        return meteofrance::download_grib(grib, progress_callback);
    } else {
        Err(format!("Downloader failed: unexpected model: {}", grib.model))
    }*/
}

