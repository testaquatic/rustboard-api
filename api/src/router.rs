use axum::{
    Router, middleware,
    routing::{get, patch, post},
};
use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handler::{
        auth::{AuthOpenApiDoc, login, me, signup},
        comment::{CommentOpenApiDoc, create_comment, list_comments},
        meta::{MetaOpenApiDoc, health, version},
        post::{PostOpenApiDoc, create_post, delete_post, get_post, list_posts, update_post},
        ws::{WsOpenApiDoc, ws_notifications},
    },
    middleware::auth::require_auth,
    state::AppState,
};

/// 인증 없이 접근 가능한 라우트
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/posts", get(list_posts))
        .route("/posts/{id}", get(get_post))
        .route("/posts/{post_id}/comments", get(list_comments))
}

/// 인증이 필수인 라우트
pub fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/posts", post(create_post))
        .route("/posts/{id}", patch(update_post).delete(delete_post))
        .route("/posts/{post_id}/comments", post(create_comment))
        .route("/me", get(me))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

/// ws 라우트
pub fn ws_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ws/notifications", get(ws_notifications))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

pub fn get_swagger_router(app_state: AppState) -> axum::Router {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let mut api = OpenApiBuilder::new()
        .info(
            Info::builder()
                .title(app_state.app_info.service_name.as_str())
                .version(VERSION)
                .description(Some(format!("{} swagger", app_state.app_info.service_name)))
                .build(),
        )
        .build();

    api.merge(MetaOpenApiDoc::openapi());
    api.merge(PostOpenApiDoc::openapi());
    api.merge(CommentOpenApiDoc::openapi());
    api.merge(AuthOpenApiDoc::openapi());
    api.merge(WsOpenApiDoc::openapi());

    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api)
        .into()
}

pub fn create_router(state: AppState) -> Router {
    // 라우터를 만들고 상태 붙이기
    Router::new()
        .merge(public_routes())
        .merge(protected_routes(state.clone()))
        .merge(ws_routes(state.clone()))
        .with_state(state.clone())
        .merge(get_swagger_router(state))
}
