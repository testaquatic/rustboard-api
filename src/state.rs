use std::sync::Arc;

use crate::service::post::PostService;

/// AppState 정의
#[derive(Clone)]
pub struct AppState {
    pub post_service: Arc<PostService>,
}
