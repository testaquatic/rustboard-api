use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;

use crate::{domain::user::User, repository::error::RepositoryError};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, display_name, role, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// 사용자의 비밀번호 해시를 변경한다.
    ///
    /// # 인자
    ///
    /// * `id` - 유저의 아이디
    /// * `new_hash` - 변경할 비밀번호 해시
    ///
    /// # 반환
    ///
    /// * `Ok(i64)` - 변경된 유저의 아이디
    /// * `Err(RepositoryError)` - 오류
    pub async fn update_password_hash(
        &self,
        id: i64,
        new_hash: &SecretString,
    ) -> Result<i64, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        // 유저가 존재하는지 확인
        let user_id = sqlx::query!("SELECT id FROM users WHERE id = $1", id)
            .fetch_optional(tx.as_mut())
            .await?;
        let Some(user_id) = user_id else {
            return Err(RepositoryError::NotFound {
                entity: "user".to_string(),
                id,
            });
        };

        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = now()
            WHERE id = $2
            "#,
            new_hash.expose_secret(),
            id
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(user_id.id)
    }

    /// 사용자를 추가한다.
    pub async fn add_user(
        &self,
        email: &str,
        password_hash: &SecretString,
        display_name: &str,
        role: &str,
    ) -> Result<User, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, display_name, role)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, password_hash, display_name, role, created_at, updated_at
            "#,
            email,
            password_hash.expose_secret(),
            display_name,
            role,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
