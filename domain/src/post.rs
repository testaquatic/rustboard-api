use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
