use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, query_as};

use crate::{
    domain::post::{CreatePostInput, Post},
    repository::error::RepositoryError,
};

#[async_trait]
pub trait PostRepository {
    async fn insert(&self, input: CreatePostInput, author_id: i64)
    -> Result<Post, RepositoryError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError>;
    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        body: Option<&str>,
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
    async fn insert(
        &self,
        input: CreatePostInput,
        author_id: i64,
    ) -> Result<Post, RepositoryError> {
        let row = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, body, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, body, author_id, created_at, updated_at
            "#,
            input.title,
            input.content,
            author_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        let row = sqlx::query_as!(
            Post,
            r#"
            SELECT id, title, body, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

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
                    SELECT id, title, body, author_id, created_at, updated_at
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
                    SELECT id, title, body, author_id, created_at, updated_at
                    FROM posts
                    ORDER BY created_at DESC, id DESC
                    LIMIT $1
                    "#,
                    limit as i64
                )
                .fetch_all(&self.pool)
                .await
            }
        }?;

        Ok(rows)
    }

    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<Option<Post>, RepositoryError> {
        let row = query_as!(
            Post,
            r#"
            UPDATE posts
            SET title = COALESCE($2, title),
                body = COALESCE($3, body),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, body, author_id, created_at, updated_at
            "#,
            id,
            title,
            body,
        )
        .fetch_optional(&self.pool)
        .await?;

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
        .await?;

        Ok(result.rows_affected() == 1)
    }
}
