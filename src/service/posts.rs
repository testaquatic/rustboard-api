use std::sync::Arc;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    domain::posts::{CreatePostInput, Post, UpdatePostInput},
    repository::posts::PostRepository,
};

pub type DynPostRepository = Arc<dyn PostRepository + Send + Sync>;

const TITLE_MAX: usize = 200;
const BODY_MAX: usize = 10_000;

pub struct PostService {
    repo: DynPostRepository,
}

impl PostService {
    pub fn new(repo: DynPostRepository) -> Self {
        Self { repo }
    }

    pub async fn create(&self, input: CreatePostInput) -> Result<Post, ServiceError> {
        let title = input.title.trim();
        if title.is_empty() {
            return Err(ServiceError::EmptyTitle);
        }
        if title.chars().count() > TITLE_MAX {
            return Err(ServiceError::TitleTooLong(TITLE_MAX));
        }
        if input.body.chars().count() > BODY_MAX {
            return Err(ServiceError::BodyTooLong(BODY_MAX));
        }

        let clean = CreatePostInput {
            title: title.to_string(),
            body: input.body,
        };
        self.repo
            .insert(clean)
            .await
            .map_err(|_| ServiceError::Internal)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Post, ServiceError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|_| ServiceError::Internal)?
            .ok_or(ServiceError::NotFound(id))
    }

    pub async fn list_recent(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, ServiceError> {
        self.repo
            .list(cursor, limit)
            .await
            .map_err(|_| ServiceError::Internal)
    }

    /// 게시글을 수정한다.
    pub async fn update(&self, id: i64, input: UpdatePostInput) -> Result<Post, ServiceError> {
        if input.title.is_none() && input.body.is_none() {
            return Err(ServiceError::EmptyTitle);
        }
        if let Some(title) = &input.title {
            let trimmed = title.trim();
            if trimmed.is_empty() {
                return Err(ServiceError::EmptyTitle);
            }
            if trimmed.chars().count() > TITLE_MAX {
                return Err(ServiceError::TitleTooLong(TITLE_MAX));
            }
        }
        if let Some(body) = &input.body
            && body.chars().count() > BODY_MAX
        {
            return Err(ServiceError::BodyTooLong(BODY_MAX));
        }

        self.repo
            .update(id, input)
            .await
            .map_err(|_| ServiceError::Internal)?
            .ok_or(ServiceError::NotFound(id))
    }

    /// 게시글을 삭제한다.
    pub async fn delete(&self, id: i64) -> Result<(), ServiceError> {
        let removed = self
            .repo
            .delete(id)
            .await
            .map_err(|_| ServiceError::Internal)?;
        if !removed {
            return Err(ServiceError::NotFound(id));
        }

        Ok(())
    }
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
pub struct ErrorBody {
    message: String,
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
                "internal error".to_string(),
            ),
        };

        (status, Json(ErrorBody { message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::repository::posts::InMemoryPostRepository;

    use super::*;

    fn make_service() -> PostService {
        let repo = Arc::new(InMemoryPostRepository::new());
        PostService::new(repo)
    }

    #[tokio::test]
    async fn 제목이_비면_에러() {
        let service = make_service();
        let result = service
            .create(CreatePostInput {
                title: "   ".into(),
                body: "본문".into(),
            })
            .await;
        assert!(matches!(result, Err(ServiceError::EmptyTitle)));
    }

    #[tokio::test]
    async fn 생성_후_조회() {
        let service = make_service();
        let created = service
            .create(CreatePostInput {
                title: "제목".into(),
                body: "본문".into(),
            })
            .await
            .expect("생성 성공");
        let fetched = service.get_by_id(created.id).await.expect("조회 성공");
        assert_eq!(fetched.title, "제목");
        assert_eq!(fetched.body, "본문");
    }
}
