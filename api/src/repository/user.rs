use rustboard_domain::{error::repository_error::RepositoryError, user::User};
use sqlx::PgPool;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UserRepository {
    pub async fn insert(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
                INSERT INTO users (email, password_hash, display_name) 
                VALUES ($1, $2, $3) 
                RETURNING id, email, password_hash, display_name, role, created_at
            "#,
            email,
            password_hash,
            display_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, display_name, role, created_at
                FROM users
                WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, email, password_hash, display_name, role, created_at
                FROM users
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}
