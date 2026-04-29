use std::sync::Arc;

use crate::{
    domain::post::{CreatePostInput, Post, ServiceError},
    repository::post::PostRepository,
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
            .map_err(|_| ServiceError::NotFound(id))?
            .ok_or(ServiceError::NotFound(id))
    }

    pub async fn list(&self) -> Result<Vec<Post>, ServiceError> {
        self.repo.list().await.map_err(|_| ServiceError::Internal)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::repository::post::InMemoryPostRepository;

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
