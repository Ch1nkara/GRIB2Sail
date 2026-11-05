use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use clap::ValueEnum;
use grib2sail as g2s;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr};

#[derive(Clone)]
struct AppState;

#[derive(Deserialize)]
struct DownloadRequest {
    model: String,
    step: String,
    days: u32,
    latitude_min: f64,
    latitude_max: f64,
    longitude_min: f64,
    longitude_max: f64,
    components: Vec<String>,
    auth: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/download", post(download))
        .with_state(AppState);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn download(
    State(_state): State<AppState>,
    Json(payload): Json<DownloadRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let model = parse_value_enum::<g2s::Model>(&payload.model, "model")?;
    let step = parse_value_enum::<g2s::Step>(&payload.step, "step")?;
    let components = payload
        .components
        .iter()
        .map(|value| parse_value_enum::<g2s::Component>(value, "components"))
        .collect::<Result<Vec<_>, _>>()?;

    let grib = g2s::Grib {
        model: model.clone(),
        step,
        days: payload.days,
        latitude_min: payload.latitude_min,
        latitude_max: payload.latitude_max,
        longitude_min: payload.longitude_min,
        longitude_max: payload.longitude_max,
        components,
    };

    let options = g2s::DownloadOptions::new().with_auth(payload.auth);
    let (content, run) = g2s::download_grib(grib, options)
        .await
        .map_err(|e| bad_request(e.to_string()))?;

    let filename = format!("{}_{}_{}.grib2", model, run, step);
    let mut response = (StatusCode::OK, content).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .map_err(|e| bad_request(e.to_string()))?,
    );
    Ok(response)
}

fn parse_value_enum<T>(value: &str, field: &str) -> Result<T, (StatusCode, Json<ErrorResponse>)>
where
    T: ValueEnum,
{
    T::from_str(value, true).map_err(|_| bad_request(format!("invalid {}: {}", field, value)))
}

fn bad_request(message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse { error: message }),
    )
}
