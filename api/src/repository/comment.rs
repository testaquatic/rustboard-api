use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rustboard_domain::{
    comment::{Comment, CreateCommentInput},
    error::RepositoryError,
    repository::CommentRepository,
};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::repository::posts::InMemoryPostRepository;

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
        // 일단 글이 있는지 확인
        let post_author_id = sqlx::query!(
            r#"
            SELECT author_id
            FROM posts
            WHERE id = $1
            "#,
            post_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| RepositoryError::NotFound {
            entity: "post",
            id: post_id,
        })?
        .author_id;

        let row = sqlx::query!(
            r#"
            INSERT INTO comments (post_id, body)
            VALUES ($1, $2)
            RETURNING 
                id, 
                post_id,
                body, 
                created_at, 
                updated_at
            "#,
            post_id,
            input.body,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Comment {
            id: row.id,
            post_id,
            post_author_id,
            body: row.body,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError> {
        let row = sqlx::query_as!(
            Comment,
            r#"
            SELECT 
                id, post_id, (SELECT author_id FROM posts WHERE id = $1) AS "post_author_id!", body, created_at, updated_at
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
            SELECT 
                id, post_id, (SELECT author_id FROM posts WHERE id = post_id) AS "post_author_id!", body, created_at, updated_at
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
    comment_repo: Arc<InMemoryPostRepository>,
}

impl InMemoryCommentRepository {
    pub fn new(comment_repo: Arc<InMemoryPostRepository>) -> Self {
        Self {
            inner: Mutex::new(InMemoryCommentState {
                next_id: 1,
                items: HashMap::new(),
            }),
            comment_repo,
        }
    }
}

#[async_trait]
impl CommentRepository for InMemoryCommentRepository {
    async fn insert(
        &self,
        post_id: i64,
        input: CreateCommentInput,
    ) -> Result<Comment, RepositoryError> {
        let post_author_id = self
            .comment_repo
            .lock()
            .await
            .items
            .get(&post_id)
            .ok_or_else(|| RepositoryError::NotFound {
                entity: "게시글이 존재하지 않습니다",
                id: post_id,
            })?
            .author_id;

        let mut state = self.inner.lock().await;
        let id = state.next_id;
        state.next_id += 1;
        let comment = Comment {
            id,
            post_id,
            body: input.body,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            post_author_id,
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
