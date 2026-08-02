use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateCommentInput {
    pub body: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentResponse {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Comment> for CommentResponse {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id,
            post_id: comment.post_id,
            body: comment.body,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }
    }
}
