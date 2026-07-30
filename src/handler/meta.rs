use axum::Json;
use serde::Serialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "rustboard-api",
    })
}

#[derive(Serialize)]
pub struct VersionResponse {
    service: &'static str,
    version: &'static str,
}

pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        service: "rustboard-api",
        version: VERSION,
    })
}
