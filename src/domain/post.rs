use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 저장한 글
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePostInput {
    /// 제목,
    /// 내용이 없거나 공백 문자만 있다면 오류,
    /// 최대 200자
    pub title: String,
    /// 본문,
    /// 최대 10000자
    pub body: String,
}

/// 서비스 계층에서 반환하는 오류
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

/// 오류일 경우의 응답
#[derive(Serialize, ToSchema)]
pub struct ErrorBody<'a> {
    message: &'a str,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ServiceError::EmptyTitle
            | ServiceError::TitleTooLong(_)
            | ServiceError::BodyTooLong(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServiceError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "내부 오류".to_string()),
        };

        (status, Json(ErrorBody { message: &message })).into_response()
    }
}
