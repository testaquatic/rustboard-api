use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rustboard_domain::user::User;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::repository::{error::RepositoryError, types::UserRepository};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub type DynUserRepository = Arc<dyn UserRepository + Send + Sync>;

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn insert(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User, RepositoryError> {
        sqlx::query_as!(
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
        .await
        .map_err(From::from)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, display_name, role, created_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(From::from)
    }
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepositoryError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, display_name, role, created_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(From::from)
    }
}

#[derive(Default)]
pub struct InMemoryUserState {
    next_id: i64,
    items: HashMap<i64, User>,
    email_index: HashMap<String, i64>,
}

pub struct InMemoryUserRepository {
    inner: Mutex<InMemoryUserState>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryUserState {
                next_id: 1,
                items: HashMap::new(),
                email_index: HashMap::new(),
            }),
        }
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn insert(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User, RepositoryError> {
        let mut state = self.inner.lock().await;

        // 이메일 중복 검사
        if state.email_index.contains_key(email) {
            // 아무 오류나 가져다 붙였다.
            return Err(RepositoryError::Query(sqlx::Error::InvalidArgument(
                "이메일이 이미 있습니다".to_string(),
            )));
        }

        let id = state.next_id;
        state.next_id += 1;

        let user = User {
            id,
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            display_name: display_name.to_string(),
            role: "user".to_string(),
            created_at: chrono::Utc::now(),
        };

        state.email_index.insert(email.to_string(), id);
        state.items.insert(id, user.clone());

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let state = self.inner.lock().await;
        let user = state
            .email_index
            .get(email)
            .and_then(|id| state.items.get(id))
            .cloned();

        Ok(user)
    }
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepositoryError> {
        let state = self.inner.lock().await;
        let user = state.items.get(&id).cloned();

        Ok(user)
    }
}
