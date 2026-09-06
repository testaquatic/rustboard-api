use chrono::{DateTime, Utc};
use secrecy::SecretString;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    #[sqlx(try_from = "String")]
    pub password_hash: SecretString,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct LoginInput {
    pub email: String,
    pub password: SecretString,
}
