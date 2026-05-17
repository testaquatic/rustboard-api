use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{const_val::PKG_VERSION, state::AppState};

/// 헬스체크 응답
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// 응답 상태
    #[schema(example = "ok")]
    status: &'static str,
    #[schema(example = "rustboard-api")]
    /// PKG_NAME을 출력한다.
    service: String,
}

/// 헬스체크 엔드포인트
#[utoipa::path(
    get,
    tag = "meta",
    path = "/health", 
    summary = "헬스체크",  
    description = "헬스체크를 담당하는 엔드포인트",
    responses((
        status = 200,
        description = "정상",
        content_type = "application/json",
        body = HealthResponse,
        example = json!(HealthResponse {
            status: "ok", 
            service: "rustboard-api".to_string() 
        })
    ))
)]
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name.clone(),
    })
}
#[derive(utoipa::OpenApi)]
#[openapi(paths(health), components(schemas(HealthResponse)))]
pub struct HealthOpenApi;

/// 버전 응답
#[derive(Serialize, ToSchema)]
pub struct VersionResponse {
    /// 서비스명
    service: String,
    /// 버전
    version: &'static str,
}

/// 버전 엔드포인트
#[utoipa::path(
    get,
    tag = "meta",
    path = "/version",
    summary = "버전",
    description = "버전을 반환하는 엔드포인트",
    responses((
        status = 200,
        content_type = "application/json",
        body = VersionResponse,
        example = json!(
            VersionResponse {
                service: "rustboard-api".to_string(),
                version: PKG_VERSION,
            }
        )
    ))
)]
pub async fn version(State(state): State<AppState>) -> impl IntoResponse {
    Json(VersionResponse {
        service: state.config.service_name.clone(),
        version: PKG_VERSION,
    })
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(version), components(schemas(VersionResponse)))]
pub struct VersionOpenApi;

#[cfg(test)]
mod tests {}
