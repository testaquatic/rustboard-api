use chrono::{DateTime, Utc};

use crate::{
    domain::{
        post::{CreatePostInput, Post, UpdatePostInput},
        role::Role,
    },
    repository::post::DynPostRepository,
    service::{check_ownership, error::ServiceError},
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

    pub async fn create(
        &self,
        input: CreatePostInput,
        author_id: i64,
    ) -> Result<Post, ServiceError> {
        let title = input.title.trim();
        if title.is_empty() {
            return Err(ServiceError::Validation("제목이 비어 있습니다".to_string()));
        }
        if title.chars().count() > TITLE_MAX {
            return Err(ServiceError::Validation(format!(
                "제목이 {}자를 초과했습니다",
                TITLE_MAX
            )));
        }
        if input.content.chars().count() > BODY_MAX {
            return Err(ServiceError::Validation(format!(
                "본문이 {}자를 초과했습니다",
                BODY_MAX
            )));
        }

        let clean = CreatePostInput {
            title: title.to_string(),
            content: input.content,
        };

        Ok(self.repo.insert(clean, author_id).await?)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Post, ServiceError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound { entity: "post", id })
    }

    pub async fn list_recent(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, ServiceError> {
        let posts = self.repo.list(cursor, limit).await?;

        Ok(posts)
    }

    pub async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
        requester_id: i64,
        requester_role: &Role,
    ) -> Result<Post, ServiceError> {
        if input.title.is_none() && input.content.is_none() {
            return Err(ServiceError::Validation(
                "수정할 내용이 없습니다".to_string(),
            ));
        }
        if let Some(title) = &input.title {
            let trimmed = title.trim();
            if trimmed.is_empty() {
                return Err(ServiceError::Validation("제목이 비어 있습니다".to_string()));
            }
            if trimmed.chars().count() > TITLE_MAX {
                return Err(ServiceError::Validation(format!(
                    "제목이 {}자를 초과했습니다",
                    TITLE_MAX
                )));
            }
        }
        if let Some(body) = &input.content
            && body.chars().count() > BODY_MAX
        {
            return Err(ServiceError::Validation(format!(
                "본문이 {}자를 초과했습니다",
                BODY_MAX
            )));
        }

        let post = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound { entity: "post", id })?;

        // 소유권 검사: 글 작성자가 아니거나 어드민이 아니면 금지
        check_ownership(post.author_id, requester_id, requester_role)?;

        self.repo
            .update(id, input.title.as_deref(), input.content.as_deref())
            .await?
            .ok_or(ServiceError::NotFound { entity: "post", id })
    }

    pub async fn delete(
        &self,
        id: i64,
        requester_id: i64,
        requester_role: &Role,
    ) -> Result<(), ServiceError> {
        let post = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound { entity: "post", id })?;

        // 본인 또는 어드민만 삭제 가능
        check_ownership(post.author_id, requester_id, requester_role)?;

        self.repo.delete(id).await?;

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
            .create(
                CreatePostInput {
                    title: "     ".into(),
                    content: "본문".into(),
                },
                1,
            )
            .await;

        assert!(matches!(result, Err(ServiceError::Validation(_))));
    }

    #[tokio::test]
    async fn get_post_by_id_after_creation() {
        let service = make_service().await;
        let created = service
            .create(
                CreatePostInput {
                    title: "첫 글".into(),
                    content: "안녕".into(),
                },
                1,
            )
            .await
            .expect("생성 성공");
        let found = service.get_by_id(created.id).await.expect("조회 성공");

        assert_eq!(found.title, "첫 글");
    }
}
