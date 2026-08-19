use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::{
    routes::{
        auth::{login, me, signup},
        comment::{create_comment, list_comments},
        meta::{health, version},
        post::{create_post, delete_post, get_post, list_posts, update_post},
    },
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
pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/posts", post(create_post))
        .route("/posts/{id}", patch(update_post).delete(delete_post))
        .route("/posts/{post_id}/comments", post(create_comment))
        .route("/me", get(me))
}
