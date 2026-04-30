use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::domain::post::{CreatePostInput, Post};

/// 저장소 트레이트
#[async_trait]
pub trait PostRepository {
    /// 저장소에 글을 저장한다.
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError>;
    /// id를 기준으로 글을 불러온다.
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    /// 모든 글을 불러온다.
    async fn list(&self) -> Result<Vec<Post>, RepositoryError>;
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
        };
        state.items.insert(id, post.clone());

        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        Ok(state.items.get(&id).cloned())
    }

    async fn list(&self) -> Result<Vec<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        let mut item = state.items.values().cloned().collect::<Vec<_>>();
        item.sort_by_key(|p| p.id);
        Ok(item)
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
            RETURNING id, title, body
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
            SELECT id, title, body
            FROM posts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
    }

    async fn list(&self) -> Result<Vec<Post>, RepositoryError> {
        sqlx::query_as!(
            Post,
            r#"
            SELECT id, title, body
            FROM posts
            ORDER BY id DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)
    }
}
