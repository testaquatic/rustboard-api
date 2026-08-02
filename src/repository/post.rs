use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query_as};
use thiserror::Error;

use crate::domain::post::{CreatePostInput, Post, UpdatePostInput};

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("저장소 오류")]
    Backend,
}

#[async_trait]
pub trait PostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError>;
    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError>;
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub type DynPostRepository = Arc<dyn PostRepository + Send + Sync>;

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError> {
        let row = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, body)
            VALUES ($1, $2)
            RETURNING id, title, body, created_at, updated_at
            "#,
            input.title,
            input.body,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)?;

        Ok(row)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        let row = sqlx::query_as!(
            Post,
            r#"
            SELECT id, title, body, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)?;

        Ok(row)
    }

    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError> {
        let rows = match cursor {
            Some((ts, id)) => {
                sqlx::query_as!(
                    Post,
                    r#"
                    SELECT id, title, body, created_at, updated_at
                    FROM posts
                    WHERE (created_at, id) < ($1, $2)
                    ORDER BY created_at DESC, id DESC
                    LIMIT $3
                    "#,
                    ts,
                    id,
                    limit as i64,
                )
                .fetch_all(&self.pool)
                .await
            }

            None => {
                sqlx::query_as!(
                    Post,
                    r#"
                    SELECT id, title, body, created_at, updated_at
                    FROM posts
                    ORDER BY created_at DESC, id DESC
                    LIMIT $1
                    "#,
                    limit as i64
                )
                .fetch_all(&self.pool)
                .await
            }
        };

        rows.map_err(|_| RepositoryError::Backend)
    }

    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError> {
        let row = query_as!(
            Post,
            r#"
            UPDATE posts
            SET title = COALESCE($2, title),
                body = COALESCE($3, body),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, body, created_at, updated_at
            "#,
            id,
            input.title,
            input.body,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM posts
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)?;

        Ok(result.rows_affected() == 1)
    }
}
