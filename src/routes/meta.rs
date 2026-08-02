use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::state::AppState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[utoipa::path(
    description = "작동상태를 확인한다.",
    get,
    path = "/health",
    responses(
        (status = 200, description = "ok", body = HealthResponse, example = json!(HealthResponse{
            status: "ok",
            service: "rustboard-api",
        }))
    ),
    tags=["meta"]
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "rustboard-api",
    })
}

#[derive(Serialize, ToSchema)]
pub struct VersionResponse {
    service: String,
    version: &'static str,
}

#[utoipa::path(
    description = "서비스 버전을 확인한다.",
    get,
    path = "/version",
    responses(
        (status = 200, description = "ok", body = VersionResponse, example = json!(VersionResponse{
            service: "rustboard-api".to_string(),
            version: VERSION,
        }))
    ),
    tags=["meta"]
)]
pub async fn version(State(state): State<AppState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        service: state.configuration.service_name.clone(),
        version: VERSION,
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(health, version),
    tags((name = "meta", description = "메타 API")),
    components(schemas(HealthResponse, VersionResponse))
)]
pub struct MetaOpenApiDoc;
