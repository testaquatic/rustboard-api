use sqlx::PgPool;

use crate::{
    domain::comment::{Comment, CreateCommentInput},
    repository::error::RepositoryError,
};

#[derive(Clone)]
pub struct CommentRepository {
    pool: PgPool,
}

impl CommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(
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
            input.body,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError> {
        let rows = sqlx::query_as!(
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

        Ok(rows)
    }

    // async fn find_by_id(&self, id: i64) -> Result<Option<Comment>, RepositoryError> {
    //     let row = sqlx::query_as!(
    //         Comment,
    //         r#"
    //         SELECT id, post_id, body, created_at, updated_at
    //         FROM comments
    //         WHERE id = $1
    //         "#,
    //         id
    //     )
    //     .fetch_optional(&self.pool)
    //     .await?;

    //     Ok(row)
    // }
}
