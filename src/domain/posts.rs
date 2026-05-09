use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 저장한 글
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 게시글을 생성 요청할 때의 json
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePostInput {
    /// 제목,
    /// 내용이 없거나 공백 문자만 있다면 오류,
    /// 최대 200자
    pub title: String,
    /// 본문,
    /// 최대 10000자
    pub content: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Post> for PostResponse {
    fn from(p: Post) -> Self {
        Self {
            id: p.id,
            title: p.title,
            body: p.body,
            author_id: p.author_id,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

/// 게시글을 수정 요청할 때의 json
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdatePostInput {
    pub title: Option<String>,
    pub body: Option<String>,
}
