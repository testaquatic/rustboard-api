use chrono::{DateTime, Utc};
use rustboard_domain::{comment::Comment, post::Post, user::User};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentResponse {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Comment> for CommentResponse {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id,
            post_id: comment.post_id,
            body: comment.body,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        PostResponse {
            id: post.id,
            title: post.title,
            body: post.body,
            author_id: post.author_id,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}
