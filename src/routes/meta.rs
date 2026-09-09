use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    service: String,
}

#[utoipa::path(
    description = "작동상태를 확인한다.",
    get,
    path = "/health",
    responses(
        (status = 200, description = "ok", body = HealthResponse, example = json!(HealthResponse{
            status: "ok",
            service: "rustboard-api".to_string(),
        }))
    ),
    tags=["meta"]
)]
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    if sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_err()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "db_unavailable",
                service: state.app_info.service_name.clone(),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            service: state.app_info.service_name.clone(),
        }),
    )
}

#[derive(Serialize, ToSchema)]
pub struct VersionResponse {
    service: String,
    version: String,
}

#[utoipa::path(
    description = "서비스 버전을 확인한다.",
    get,
    path = "/version",
    responses(
        (status = 200, description = "ok", body = VersionResponse, example = json!(VersionResponse{
            service: "rustboard-api".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    ),
    tags=["meta"]
)]
pub async fn version(State(state): State<AppState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        service: state.app_info.service_name.clone(),
        version: state.app_info.service_version.clone(),
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(health, version),
    tags((name = "meta", description = "메타 API")),
    components(schemas(HealthResponse, VersionResponse))
)]
pub struct MetaOpenApiDoc;
