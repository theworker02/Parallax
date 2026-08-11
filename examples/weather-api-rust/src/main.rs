//! Parallax-migrated entrypoint.

mod app;
mod routes;
mod service;
mod types;

use axum::{extract::Path, routing::get, Json, Router};
use std::net::SocketAddr;

async fn weather_handler(Path(city): Path<String>) -> Json<serde_json::Value> {
    let weather = service::get_weather(&city);
    let temperature_f = service::celsius_to_fahrenheit(weather.temperature_c);
    Json(serde_json::json!({
        "city": weather.city,
        "temperatureC": weather.temperature_c,
        "conditions": weather.conditions,
        "temperatureF": temperature_f,
    }))
}

async fn forecast_handler(Path(city): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(service::get_forecast(&city)).unwrap_or_default())
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/weather/{city}", get(weather_handler))
        .route("/forecast/{city}", get(forecast_handler));
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
