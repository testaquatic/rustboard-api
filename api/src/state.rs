use std::sync::Arc;

use rustboard_domain::configuration::Settings;
use secrecy::SecretString;

use crate::{
    client::notification::NotificationClient,
    service::{comment::CommentService, post::PostService, user::UserService},
};

/// AppState 정의
#[derive(Clone)]
pub struct AppState {
    pub app_info: Arc<AppInfo>,
    pub pool: sqlx::PgPool,
    pub post_service: Arc<PostService>,
    pub comment_service: Arc<CommentService>,
    pub user_service: Arc<UserService>,
    pub notification_client: NotificationClient,
}

#[derive(Clone)]
pub struct AppInfo {
    pub service_name: String,
    pub service_version: String,
    pub jwt_secret: SecretString,
    pub jwt_expiration_minutes: i64,
}

impl From<Settings> for AppInfo {
    fn from(settings: Settings) -> Self {
        Self {
            service_name: settings.service_name,
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            jwt_secret: SecretString::new(settings.jwt_secret.into_boxed_str()),
            jwt_expiration_minutes: settings.jwt_expiration_minutes,
        }
    }
}
