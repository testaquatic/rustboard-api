use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
