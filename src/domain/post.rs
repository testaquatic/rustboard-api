use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePostInput {
    pub title: String,
    pub body: String,
}

#[derive(Debug, thiserror::Error)]
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
