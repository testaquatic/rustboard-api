use std::sync::Arc;

use axum::http::{Request, StatusCode};
use rustboard_api::{
    config::Config, repository::post::InMemoryPostRepository, router::app_routes,
    service::post::PostService, state::AppState,
};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_without_db() {
    let repo = Arc::new(InMemoryPostRepository::new());
    let state = AppState {
        config: Arc::new(Config::from_env().unwrap()),
        posts_service: Arc::new(PostService::new(repo)),
    };

    let app = app_routes(&state.config).with_state(state);

    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
