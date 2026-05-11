use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 댓글 데이터 모델
#[derive(Clone)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
/// 코멘트를 생성할 때 사용하는 데이터 모델
pub struct CreateCommentInput {
    pub body: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommentResponse {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Comment> for CommentResponse {
    fn from(c: Comment) -> Self {
        Self {
            id: c.id,
            post_id: c.post_id,
            body: c.body,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
