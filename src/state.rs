use std::sync::Arc;

use crate::{
    configuration::Settings,
    service::{comment::CommentService, post::PostService, user::UserService},
};

/// AppState 정의
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub configuration: Arc<Settings>,
    pub post_service: Arc<PostService>,
    pub comment_service: Arc<CommentService>,
    pub user_service: Arc<UserService>,
}
