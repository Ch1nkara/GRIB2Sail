use futures::{stream, StreamExt};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use tokio::sync::mpsc::UnboundedSender;

use crate::utils::config::{Grib, Urls, DownloadEvent};
use crate::meteofrance;


pub async fn download_grib(mut grib: Grib, events: UnboundedSender<DownloadEvent>)
-> Result<Grib, reqwest::Error> {
    let client = Client::new();

    let dummy_urls: Vec<String> = vec![
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
        String::from("http://jsonplaceholder.typicode.com/todos/1"),
    ];
    let mut dummy_headers = HeaderMap::new();
    dummy_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let mut urls = Urls {
        urls: dummy_urls,
        headers: dummy_headers
    };
    grib.run = Some("azerty".to_string());
/*
    if grib.model.to_string().starts_with("arome") {
         grib, urls = meteofrance::generate_urls(grib, client);
    } else {
        Err(format!("Downloader failed: unexpected model: {}", grib.model))
    }
*/
    let total = 100;

    let _ = events.send(DownloadEvent::Started {total});

    let results = stream::iter(urls.urls.clone().into_iter().enumerate())
        .map(|(idx, url)| {
            let client = &client;
            let events = events.clone();
            let cloned_urls = urls.clone();
            async move {
                let resp = client
                    .get(&url)
                    .headers(cloned_urls.headers)
                    .send()
                    .await?
                    .error_for_status()?;
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
    let mut parts: Vec<(usize, Vec<u8>)> = results.into_iter().collect::<Result<_, _>>()?;

    // Restore original order
    parts.sort_by_key(|(idx, _)| *idx);

    // Concatenate
    let mut ordered = Vec::new();
    for (_, mut data) in parts { ordered.append(&mut data) }

    //grib.content = results.into_iter().map(|v| v.unwrap()).collect();
    grib.content = Some(ordered);

    Ok(grib)
}

