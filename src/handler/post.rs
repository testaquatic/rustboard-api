use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    domain::post::{CreatePostInput, Post, ServiceError},
    state::AppState,
};

pub async fn list_posts(State(state): State<AppState>) -> Result<Json<Vec<Post>>, ServiceError> {
    let posts = state.post_service.list().await?;

    Ok(Json(posts))
}

pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Post>, ServiceError> {
    let post = state.post_service.get_by_id(id).await?;

    Ok(Json(post))
}

pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<Post>), ServiceError> {
    let post = state.post_service.create(payload).await?;

    Ok((StatusCode::CREATED, Json(post)))
}
