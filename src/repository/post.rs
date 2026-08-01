use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, query_as};
use thiserror::Error;
use tokio::sync::Mutex;

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
    async fn list(&self) -> Result<Vec<Post>, RepositoryError>;
    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError>;
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError> {
        let mut state = self.inner.lock().await;
        let post = state.items.get_mut(&id);

        match post {
            Some(post) => {
                if let Some(title) = input.title {
                    post.title = title;
                }
                if let Some(body) = input.body {
                    post.body = body;
                }
                post.updated_at = Utc::now();
                Ok(Some(post.clone()))
            }
            None => Ok(None),
        }
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

    async fn list(&self) -> Result<Vec<Post>, RepositoryError> {
        let rows = sqlx::query_as!(
            Post,
            r#"
            SELECT id, title, body, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC, id DESC
            LIMIT 50
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RepositoryError::Backend)?;

        Ok(rows)
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
}
