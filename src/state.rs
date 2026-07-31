use std::sync::Arc;

use crate::{configuration::Settings, service::post::PostService};

/// AppState 정의
#[derive(Clone)]
pub struct AppState {
    pub configuration: Arc<Settings>,
    pub post_service: Arc<PostService>,
}
