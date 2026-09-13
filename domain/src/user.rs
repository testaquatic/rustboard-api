use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
