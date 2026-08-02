use chrono::{DateTime, Utc};

use crate::{
    domain::post::{CreatePostInput, Post, ServiceError, UpdatePostInput},
    repository::post::DynPostRepository,
};

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sqlx::postgres::PgPoolOptions;

    use crate::{configuration::get_configuration, repository::post::PostgresPostRepository};

    use super::*;

    async fn make_service() -> PostService {
        // 설정을 읽는다
        let configuration = Arc::new(get_configuration().expect("Failed to get configuration"));

        // DB 풀 만들기
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&configuration.database_url)
            .await
            .expect("Failed to connect database");

        let repo = Arc::new(PostgresPostRepository::new(pool));

        PostService::new(repo)
    }

    #[tokio::test]
    async fn error_if_title_is_empty() {
        let service = make_service().await;
        let result = service
            .create(CreatePostInput {
                title: "     ".into(),
                body: "본문".into(),
            })
            .await;

        assert!(matches!(result, Err(ServiceError::EmptyTitle)));
    }

    #[tokio::test]
    async fn get_post_by_id_after_creation() {
        let service = make_service().await;
        let created = service
            .create(CreatePostInput {
                title: "첫 글".into(),
                body: "안녕".into(),
            })
            .await
            .expect("생성 성공");
        let found = service.get_by_id(created.id).await.expect("조회 성공");

        assert_eq!(found.title, "첫 글");
    }
}
