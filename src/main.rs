use axum::{Json, response::IntoResponse, routing::get};
use serde::Serialize;
use utoipa::{
    OpenApi, ToSchema,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 헬스체크 응답
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// 응답 상태
    #[schema(example = "ok")]
    status: &'static str,
    #[schema(example = "rustboard-api")]
    /// PKG_NAME을 출력한다.
    service: &'static str,
}

/// 헬스체크 엔드포인트
#[utoipa::path(get, path = "/health", summary = "헬스체크",  description = "헬스체크를 담당하는 엔드포인트",
    responses(
        (
            status = 200, description = "정상", body = HealthResponse, 
            example = json!(HealthResponse { status: "ok", service: PKG_NAME })
        )
    )
)]
async fn heath() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: PKG_NAME,
    })
}
#[derive(utoipa::OpenApi)]
#[openapi(paths(heath), components(schemas(HealthResponse)))]
pub struct HealthOpenApi;

/// 버전 응답
#[derive(Serialize, ToSchema)]
pub struct VersionResponse {
    /// 서비스명
    service: &'static str,
    /// 버전
    version: &'static str,
}

/// 버전 엔드포인트
#[utoipa::path(
    get,
    path = "/version",
    summary = "버전",
    description = "버전을 반환하는 엔드포인트",
    responses(
        (
            status = 200,
            description = "정상",
            body = VersionResponse,
            example = json!(VersionResponse {
                service: PKG_NAME,
                version: PKG_VERSION,
            })
        )
    )
)]
async fn version() -> impl IntoResponse {
    Json(VersionResponse {
        service: PKG_NAME,
        version: PKG_VERSION,
    })
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(version), components(schemas(VersionResponse)))]
pub struct VersionOpenApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut openapi = OpenApiBuilder::new()
        .info(Info::new(PKG_NAME, PKG_VERSION))
        .build();
    openapi.merge(HealthOpenApi::openapi());
    openapi.merge(VersionOpenApi::openapi());

    let app = axum::Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi))
        .route("/health", axum::routing::get(heath))
        .route("/version", get(version));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!(
        "rustboard-api listening on http://{}",
        listener.local_addr()?
    );

    axum::serve(listener, app).await?;

    Ok(())
}
