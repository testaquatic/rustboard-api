use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::domain::posts::{CreatePostInput, Post, UpdatePostInput};

/// 저장소 트레이트
#[async_trait]
pub trait PostRepository {
    /// 저장소에 글을 저장한다.
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError>;
    /// id를 기준으로 글을 불러온다.
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    /// 모든 글을 불러온다.
    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError>;
    /// 게시글을 수정한다.
    /// 해당 글이 없으면 Result<None>을 반환한다.
    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError>;
    /// 게시글을 삭제한다.
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

/// 저장소 계층에서 반환하는 오류
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("저장소 오류")]
    Backend,
}

/// 메모리 저장소
#[derive(Default)]
struct InMemoryState {
    next_id: i64,
    items: HashMap<i64, Post>,
}

#[derive(Default)]
pub struct InMemoryPostRepository {
    inner: Mutex<InMemoryState>,
}

impl InMemoryPostRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryState::default()),
        }
    }
}

#[async_trait]
impl PostRepository for InMemoryPostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError> {
        let mut state = self.inner.lock().await;
        let id = state.next_id;
        state.next_id += 1;
        let post = Post {
            id,
            title: input.title,
            body: input.body,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        state.items.insert(id, post.clone());

        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        Ok(state.items.get(&id).cloned())
    }

    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        let mut item = state
            .items
            .iter()
            .filter_map(|(id, post)| match cursor {
                Some((cursor_ts, cursor_id)) => {
                    if post.created_at < cursor_ts && *id < cursor_id {
                        Some(post)
                    } else {
                        None
                    }
                }
                None => Some(post),
            })
            .cloned()
            .collect::<Vec<_>>();
        item.sort_by_key(|p| p.id);
        Ok(item
            .chunks(limit as usize)
            .next()
            .map(ToOwned::to_owned)
            .unwrap_or_default())
    }

    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError> {
        let mut state = self.inner.lock().await;

        // id에 해당하는 Post가 없으면 Ok(None)을 반환한다.
        let Some(post) = state.items.get_mut(&id) else {
            return Ok(None);
        };

        // post를 수정한다.
        if let Some(title) = input.title {
            post.title = title;
        }
        if let Some(body) = input.body {
            post.body = body;
        }
        post.updated_at = chrono::Utc::now();

        Ok(Some(post.clone()))
    }

    async fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let mut state = self.inner.lock().await;
        let removed = state.items.remove(&id);

        Ok(removed.is_some())
    }
}

/// Postgres 저장소
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    /// Postgres 저장소를 생성한다.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError> {
        sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, body)
            VALUES ($1, $2)
            RETURNING id, title, body, created_at, updated_at
            "#,
            input.title,
            input.body
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        sqlx::query_as!(
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
        .map_err(|_| RepositoryError::Backend)
    }

    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError> {
        match cursor {
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
                    limit as i64
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
        }
        .map_err(|_| RepositoryError::Backend)
    }

    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError> {
        sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET 
                title = COALESCE($1, title),
                body = COALESCE($2, body), 
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, title, body, created_at, updated_at
            "#,
            input.title,
            input.body,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
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
