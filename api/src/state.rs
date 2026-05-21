use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::{
    client::notification::NotificationClient,
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
    pub ws_semaphore: Arc<Semaphore>,
    pub notification_client: NotificationClient,
}
