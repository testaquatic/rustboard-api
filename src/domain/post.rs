use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePostInput {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdatePostInput {
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        PostResponse {
            id: post.id,
            title: post.title,
            body: post.body,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

#[derive(Debug, thiserror::Error, ToSchema)]
pub enum ServiceError {
    #[error("제목이 비어 있습니다")]
    EmptyTitle,
    #[error("제목이 {0}자를 초과했습니다")]
    TitleTooLong(usize),
    #[error("본문이 {0}자를 초과했습니다")]
    BodyTooLong(usize),
    #[error("게시글을 찾을 수 없습니다: {0}")]
    NotFound(i64),
    #[error("내부 오류")]
    Internal,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ServiceError::EmptyTitle
            | ServiceError::TitleTooLong(_)
            | ServiceError::BodyTooLong(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServiceError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            ),
        };

        let body = Json(json!({"error": message}));

        (status, body).into_response()
    }
}
