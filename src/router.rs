use axum::{Router, routing::get};

use crate::{
    handler::{
        meta::{health, version},
        post::{create_post, get_post, list_posts},
    },
    state::AppState,
};

pub fn app_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/{id}", get(get_post))
}
