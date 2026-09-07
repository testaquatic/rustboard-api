use chrono::{DateTime, Utc};

#[derive(Clone, serde::Serialize)]
pub struct PostRow {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PostWithAuthor {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_email: String,
    pub author_display_name: String,
}

impl From<PostWithAuthor> for PostRow {
    fn from(value: PostWithAuthor) -> Self {
        Self {
            id: value.id,
            title: value.title,
            content: value.content,
            author_id: value.author_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct PostListResponse {
    pub posts: Vec<PostRow>,
    pub total: i64,
    pub page: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreatePostInput {
    pub title: String,
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct PostResponse {
    title: String,
    content: String,
    author_id: String,
}

impl From<PostRow> for PostResponse {
    fn from(value: PostRow) -> Self {
        Self {
            title: value.title,
            content: value.content,
            author_id: value.author_id.to_string(),
        }
    }
}
