use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::comment::{Comment, CreateCommentInput},
    repository::posts::RepositoryError,
};

#[async_trait]
pub trait CommentRepository {
    async fn insert(
        &self,
        post_id: i64,
        input: CreateCommentInput,
    ) -> Result<Comment, RepositoryError>;

    async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError>;

    async fn find_by_id(&self, id: i64) -> Result<Option<Comment>, RepositoryError>;
}

pub type DynCommentRepository = Arc<dyn CommentRepository + Send + Sync>;

pub struct PostgresCommentRepository {
    pool: PgPool,
}

impl PostgresCommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentRepository for PostgresCommentRepository {
    async fn insert(
        &self,
        post_id: i64,
        input: CreateCommentInput,
    ) -> Result<Comment, RepositoryError> {
        sqlx::query_as!(
            Comment,
            r#"
            INSERT INTO comments (post_id, body)
            VALUES ($1, $2)
            RETURNING id, post_id, body, created_at, updated_at
            "#,
            post_id,
            input.body
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
    }

    async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError> {
        sqlx::query_as!(
            Comment,
            r#"
            SELECT id, post_id, body, created_at, updated_at
            FROM comments
            WHERE post_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
            post_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Comment>, RepositoryError> {
        sqlx::query_as!(
            Comment,
            r#"
            SELECT id, post_id, body, created_at, updated_at
            FROM comments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
    }
}
