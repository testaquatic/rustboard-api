use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::AppState;

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
    service: String,
    version: &'static str,
}

pub async fn version(State(state): State<AppState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        service: state.configuration.service_name.clone(),
        version: VERSION,
    })
}
