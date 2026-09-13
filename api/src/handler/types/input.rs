use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SignupInput {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateCommentInput {
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreatePostInput {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdatePostInput {
    pub title: Option<String>,
    pub content: Option<String>,
}
