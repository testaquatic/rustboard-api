use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    domain::posts::{CreatePostInput, Post, UpdatePostInput},
    repository::{error::RepositoryError, posts::PostRepository},
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
            return Err(ServiceError::Validation("제목이 비어 있습니다".to_string()));
        }
        if title.chars().count() > TITLE_MAX {
            return Err(ServiceError::Validation(format!(
                "제목이 {TITLE_MAX}자를 초과했습니다"
            )));
        }
        if input.body.chars().count() > BODY_MAX {
            return Err(ServiceError::Validation(format!(
                "본문이 {BODY_MAX}자를 초과했습니다"
            )));
        }

        let clean = CreatePostInput {
            title: title.to_string(),
            body: input.body,
        };

        self.repo.insert(clean).await.map_err(From::from)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Post, ServiceError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound {
                entity: "post".into(),
                id,
            })
    }

    pub async fn list_recent(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, ServiceError> {
        self.repo.list(cursor, limit).await.map_err(From::from)
    }

    /// 게시글을 수정한다.
    pub async fn update(&self, id: i64, input: UpdatePostInput) -> Result<Post, ServiceError> {
        if input.title.is_none() && input.body.is_none() {
            return Err(ServiceError::Validation("제목이 비어 있습니다".into()));
        }
        if let Some(title) = &input.title {
            let trimmed = title.trim();
            if trimmed.is_empty() {
                return Err(ServiceError::Validation("제목이 비어 있습니다".into()));
            }
            if trimmed.chars().count() > TITLE_MAX {
                return Err(ServiceError::Validation(format!(
                    "제목이 {TITLE_MAX}자를 초과했습니다"
                )));
            }
        }
        if let Some(body) = &input.body
            && body.chars().count() > BODY_MAX
        {
            return Err(ServiceError::Validation(format!(
                "본문이 {BODY_MAX}자를 초과했습니다"
            )));
        }

        self.repo
            .update(id, input)
            .await?
            .ok_or(ServiceError::NotFound {
                entity: "post".into(),
                id,
            })
    }

    /// 게시글을 삭제한다.
    pub async fn delete(&self, id: i64) -> Result<(), ServiceError> {
        let removed = self.repo.delete(id).await?;
        if !removed {
            return Err(ServiceError::NotFound {
                entity: "post".into(),
                id,
            });
        }

        Ok(())
    }
}

/// 서비스 계층에서 반환하는 오류
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("리포지토리 오류")]
    Repo(#[from] RepositoryError),

    #[error("입력값 검증 실패: {0}")]
    Validation(String),

    #[error("{entity}(id={id})을 찾을 수 없습니다")]
    NotFound { entity: String, id: i64 },
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
        assert!(matches!(result, Err(ServiceError::Validation(msg)) if msg.contains("비어")));
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
