use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    auth::extractor::AuthUser,
    domain::post::{CreatePostInput, PostListResponse, PostResponse},
    error::AppError,
    state::AppState,
};

pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PostResponse>, AppError> {
    let post = state.posts_service.find_by_id(id).await?;
    Ok(Json(PostResponse::from(post)))
}

#[derive(Deserialize)]
pub struct ListPostsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<PostListResponse>, AppError> {
    let posts = state.posts_service.list(query.page, query.limit).await?;
    Ok(Json(posts))
}

pub async fn create_post(
    State(state): State<AppState>,
    auth_user: AuthUser,
    create_post_input: Json<CreatePostInput>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    let post = state
        .posts_service
        .create_post(&create_post_input, auth_user.user_id)
        .await?;

    Ok((StatusCode::CREATED, Json(PostResponse::from(post))))
}
