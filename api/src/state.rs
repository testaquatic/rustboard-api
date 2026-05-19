use std::sync::Arc;

use rustboard_domain::notification::Notification;
use tokio::sync::{Semaphore, broadcast};

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
    pub notify_tx: broadcast::Sender<Notification>,
    pub ws_semaphore: Arc<Semaphore>,
}
