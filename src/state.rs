use std::sync::Arc;

use crate::{config::Config, service::post::PostService};

/// 앱 상태
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub posts_service: Arc<PostService>,
}
