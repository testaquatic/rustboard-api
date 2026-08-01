use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::domain::post::{CreatePostInput, Post};

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("저장소 오류")]
    Backend,
}

#[async_trait]
pub trait PostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    async fn list(&self) -> Result<Vec<Post>, RepositoryError>;
}

pub type DynPostRepository = Arc<dyn PostRepository + Send + Sync>;

#[derive(Default)]
struct InMemoryState {
    next_id: i64,
    items: HashMap<i64, Post>,
}

pub struct InMemoryPostRepository {
    inner: Mutex<InMemoryState>,
}

impl InMemoryPostRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryState {
                next_id: 1,
                items: HashMap::new(),
            }),
        }
    }
}

impl Default for InMemoryPostRepository {
    fn default() -> Self {
        Self::new()
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
        let post = state.items.get(&id).cloned();

        Ok(post)
    }

    async fn list(&self) -> Result<Vec<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        let mut posts = state.items.values().cloned().collect::<Vec<Post>>();
        posts.sort_by_key(|p| p.id);

        Ok(posts)
    }
}

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
        let post = sqlx::query_as!(
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
        .map_err(|_| RepositoryError::Backend)?;

        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        let post = sqlx::query_as!(
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
        .map_err(|_| RepositoryError::Backend)?;

        Ok(post)
    }

    async fn list(&self) -> Result<Vec<Post>, RepositoryError> {
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT id, title, body
            FROM posts
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)?;

        Ok(posts)
    }
}
