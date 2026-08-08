use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{domain::user::User, repository::error::RepositoryError};

#[async_trait]
pub trait UserRepository {
    async fn insert(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User, RepositoryError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepositoryError>;
}

pub type DynUserRepository = Arc<dyn UserRepository + Send + Sync>;

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn insert(
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

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
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

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepositoryError> {
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
