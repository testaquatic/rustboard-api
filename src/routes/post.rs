use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::post::{PostListResponse, PostRow},
    error::AppError,
    state::AppState,
};

#[derive(Serialize)]
pub struct PostResponse {
    title: String,
    content: String,
    author_id: String,
}

impl From<PostRow> for PostResponse {
    fn from(value: PostRow) -> Self {
        Self {
            title: value.title,
            content: value.content,
            author_id: value.author_id.to_string(),
        }
    }
}

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

pub async fn create_post() {}
