use sqlx::PgPool;

use crate::{domain::comment::CommentRow, repository::error::RepositoryError};

pub struct CommentRepository {
    pool: PgPool,
}

impl CommentRepository {
    pub async fn create_comment(
        &self,
        post_id: i64,
        content: &str,
        author_id: i64,
    ) -> Result<CommentRow, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        // 게시글 존재 확인
        let post_exist = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1) AS "exists!"
            "#,
            post_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if !post_exist {
            return Err(RepositoryError::NotFound {
                entity: "post".to_string(),
                id: post_id,
            });
        }

        let comment = sqlx::query_as!(
            CommentRow,
            r#"
            INSERT INTO comments (content, post_id, author_id)
            VALUES($1, $2, $3)
            RETURNING id, content, post_id, author_id, created_at
            "#,
            content,
            post_id,
            author_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(comment)
    }
}
