use chrono::{DateTime, Utc};

pub struct CommentRow {
    pub id: i64,
    pub content: String,
    pub post_id: i64,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
}
