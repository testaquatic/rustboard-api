use sqlx::PgPool;

use crate::{
    domain::post::{PostRow, PostWithAuthor},
    repository::error::RepositoryError,
};

pub struct PostsRepository {
    pool: PgPool,
}

impl PostsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<PostRow>, RepositoryError> {
        let post = sqlx::query_as!(
            PostRow,
            r#"SELECT id, title, content, author_id, created_at, updated_at 
            FROM posts WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn find_by_id_with_author(
        &self,
        id: i64,
    ) -> Result<Option<PostWithAuthor>, RepositoryError> {
        let row = sqlx::query_as!(
            PostWithAuthor,
            r#"
            SELECT
                p.id, p.title, p.content, p.author_id, p.created_at, p.updated_at, 
                u.email AS author_email, u.display_name AS author_display_name
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list(
        &self,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<PostWithAuthor>, i64), RepositoryError> {
        let offset = (page - 1) * limit;

        let posts = sqlx::query_as!(
            PostWithAuthor,
            r#"
            SELECT
                p.id,
                p.title,
                p.content,
                p.author_id,
                p.created_at,
                p.updated_at,
                u.email AS author_email,
                u.display_name AS author_display_name
            FROM posts p
            JOIN users u ON p.author_id = u.id
            ORDER BY p.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar(r#"SELECT COUNT(*) AS count FROM posts"#)
            .fetch_one(&self.pool)
            .await?;

        Ok((posts, total))
    }

    pub async fn create(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<PostRow, RepositoryError> {
        let post = sqlx::query_as!(
            PostRow,
            r#"
            INSERT INTO posts(title, content, author_id)
            VALUES($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
            title,
            content,
            author_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }
}
