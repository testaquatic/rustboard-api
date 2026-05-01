use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    config::Config,
    service::{comments::CommentService, posts::PostService},
};

/// 앱 상태
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub posts_service: Arc<PostService>,
    pub comments_service: Arc<CommentService>,
}
