use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    routes::{
        auth::signup,
        comment::{create_comment, list_comments},
        meta::{health, version},
        post::{create_post, delete_post, get_post, list_posts, update_post},
    },
    state::AppState,
};

pub fn app_routes() -> Router<AppState> {
    Router::new()
        .merge(meta_routes())
        .merge(posts_routes())
        .merge(comments_routes())
        .merge(auth_routes())
}

pub fn meta_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
}

pub fn posts_routes() -> Router<AppState> {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route(
            "/posts/{id}",
            get(get_post).patch(update_post).delete(delete_post),
        )
}

pub fn comments_routes() -> Router<AppState> {
    Router::new().route(
        "/posts/{post_id}/comments",
        get(list_comments).post(create_comment),
    )
}

pub fn auth_routes() -> Router<AppState> {
    Router::new().route("/signup", post(signup))
}
