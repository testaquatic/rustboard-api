use std::time::Duration;

use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use rustboard_domain::const_val::PKG_VERSION;
use serde_json::json;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    handler::{
        auth::{self, AuthOpenApi},
        comment::{CommentsOpenApi, create_comment, list_comments},
        meta::{HealthOpenApi, VersionOpenApi, health, version},
        posts::{PostsOpenApi, create_post, delete_post, get_post, list_posts, update_post},
        ws,
    },
    middleware::{
        self, auth::AuthMiddleware, rate_limit_error::rete_limit_error_response,
        rate_limit_key::ForwardedIpKeyExtractor,
    },
    state::AppState,
};

/// 최종 라우터
pub fn create_app_router_with_middleware(config: &Config, state: AppState) -> Router {
    // 동시 접속수를 제한하는 레이어
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .key_extractor(ForwardedIpKeyExtractor)
        .finish()
        .unwrap();
    let governor_layer = GovernorLayer::new(governor_conf).error_handler(rete_limit_error_response);

    create_router(config, state)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(governor_layer)
        .layer(middleware::metric::TrackMetricsLayer)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(
            middleware::request_id::add_request_id,
        ))
}

/// axum 라우터
pub fn create_router(config: &Config, state: AppState) -> Router {
    Router::new()
        // 인증 없이 접근 가능한 라우터
        .merge(public_routes())
        // 인증이 필요한 라우터
        .merge(protected_routes(state.clone()))
        // /swagger
        .merge(openapi_router(config))
        // /admin
        .merge(admin_routes())
        .with_state(state)
}

/// 인증 없이 접근 가능한 라우터
pub fn public_routes() -> Router<AppState> {
    Router::new()
        // 헬스 체크
        .route("/health", get(health))
        // 버전 체크
        .route("/version", get(version))
        // 회원 가입
        .route("/signup", post(auth::signup))
        // 로그인
        .route("/login", post(auth::login))
        // 글 목록
        .route("/posts", get(list_posts))
        // 글 조회
        .route("/posts/{id}", get(get_post))
        // 댓글 목록
        .route("/posts/{post_id}/comments", get(list_comments))
}

/// 인증이 필요한 라우터
pub fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // 글 작성
        .route("/posts", post(create_post))
        // 글 작성과 삭제
        .route("/posts/{id}", patch(update_post).delete(delete_post))
        // 댓글 작성
        .route("/posts/{post_id}/comments", post(create_comment))
        // 댓글 알림
        .route("/ws/notifications", get(ws::ws_notification))
        .route_layer(AuthMiddleware { state })
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/stats", get(admin_stats))
        .route_layer(middleware::ip_guard::AllowedIPLayer)
}

async fn admin_stats() -> impl IntoResponse {
    Json(json!({"total_posts": 43, "total_comments": 12}))
}

/// swegger 지원을 위한 라우터
fn openapi_router(config: &Config) -> SwaggerUi {
    let mut openapi = OpenApiBuilder::new()
        .info(Info::new(config.service_name.as_str(), PKG_VERSION))
        .build();
    openapi.merge(HealthOpenApi::openapi());
    openapi.merge(VersionOpenApi::openapi());
    openapi.merge(PostsOpenApi::openapi());
    openapi.merge(CommentsOpenApi::openapi());
    openapi.merge(AuthOpenApi::openapi());

    SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi)
}
