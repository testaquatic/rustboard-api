use axum::{
    Router, middleware,
    routing::{get, patch, post},
};

use crate::{
    middleware::auth::require_auth,
    routes::{
        auth::{login, me, signup},
        comment::{create_comment, list_comments},
        meta::{health, version},
        post::{create_post, delete_post, get_post, list_posts, update_post},
    },
    state::AppState,
    swagger::get_swagger_router,
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
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/posts", post(create_post))
        .route("/posts/{id}", patch(update_post).delete(delete_post))
        .route("/posts/{post_id}/comments", post(create_comment))
        .route("/me", get(me))
}

pub fn create_router(state: AppState) -> Router {
    // 라우터를 만들고 상태 붙이기
    Router::new()
        .merge(public_routes())
        .merge(
            protected_routes()
                .route_layer(middleware::from_fn_with_state(state.clone(), require_auth)),
        )
        .with_state(state.clone())
        .merge(get_swagger_router(state))
}
