use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;
use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    const_val::PKG_VERSION,
    middleware::{self, auth::AuthMiddleware},
    routes::{
        auth::{self, AuthOpenApi},
        comments::{CommentsOpenApi, create_comment, list_comments},
        meta::{HealthOpenApi, VersionOpenApi, health, version},
        posts::{PostsOpenApi, create_post, delete_post, get_post, list_posts, update_post},
    },
    state::AppState,
};

/// axum 라우터
pub fn create_router(config: &Config, state: AppState) -> Router {
    Router::new()
        // 인증 없이 접근 가능한 라우터
        .merge(public_routes())
        // 인증이 필요한 라우터
        .merge(protected_routes().route_layer(AuthMiddleware {
            state: state.clone(),
        }))
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
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        // 글 작성
        .route("/posts", post(create_post))
        // 글 작성과 삭제
        .route("/posts/{id}", patch(update_post).delete(delete_post))
        // 댓글 작성
        .route("/posts/{post_id}/comments", post(create_comment))
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
