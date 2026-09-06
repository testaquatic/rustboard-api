use std::sync::Arc;

use crate::{configuration::Settings, service::post::PostsService};

/// 애플리케이션 상태
#[derive(Clone)]
pub struct AppState {
    /// 설정
    pub configuration: Arc<Settings>,
    pub posts_service: Arc<PostsService>,
}
