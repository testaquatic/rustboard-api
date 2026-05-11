use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::{
    domain::comment::{Comment, CreateCommentInput},
    repository::error::RepositoryError,
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

    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
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
        let row = sqlx::query_as!(
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
        .await?;

        Ok(row)
    }

    async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError> {
        let row = sqlx::query_as!(
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
        .await?;

        Ok(row)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Comment>, RepositoryError> {
        let row = sqlx::query_as!(
            Comment,
            r#"
            SELECT id, post_id, body, created_at, updated_at
            FROM comments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        sqlx::query!(
            r#"
            DELETE FROM comments
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() == 1)
        .map_err(From::from)
    }
}

pub struct InMemoryCommentState {
    next_id: i64,
    items: HashMap<i64, Comment>,
}

/// 글을 잘못 따라갔는지 안 보여서 직접 구현했다.
pub struct InMemoryCommentRepository {
    inner: Mutex<InMemoryCommentState>,
}

impl InMemoryCommentRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryCommentState {
                next_id: 1,
                items: HashMap::new(),
            }),
        }
    }
}

impl Default for InMemoryCommentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommentRepository for InMemoryCommentRepository {
    async fn insert(
        &self,
        post_id: i64,
        input: CreateCommentInput,
    ) -> Result<Comment, RepositoryError> {
        let mut state = self.inner.lock().await;
        let id = state.next_id;
        state.next_id += 1;
        let comment = Comment {
            id,
            post_id,
            body: input.body,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        state.items.insert(id, comment.clone());

        Ok(comment)
    }

    async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError> {
        let state = self.inner.lock().await;
        let mut items = state
            .items
            .values()
            .filter(|comment| comment.post_id == post_id)
            .cloned()
            .collect::<Vec<_>>();

        items.sort_by(|a, b| match b.created_at.cmp(&a.created_at) {
            std::cmp::Ordering::Equal => b.id.cmp(&a.id),
            ord => ord,
        });

        Ok(items)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Comment>, RepositoryError> {
        let state = self.inner.lock().await;
        Ok(state.items.get(&id).cloned())
    }

    async fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let mut state = self.inner.lock().await;
        let removed = state.items.remove(&id);

        Ok(removed.is_some())
    }
}
