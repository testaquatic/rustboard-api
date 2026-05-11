use std::sync::Arc;

use crate::{
    config::Config,
    service::{comments::CommentService, posts::PostService, user::UserService},
};

/// 앱 상태
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub posts_service: Arc<PostService>,
    pub comments_service: Arc<CommentService>,
    pub users_service: Arc<UserService>,
}
