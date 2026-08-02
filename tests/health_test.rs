use std::sync::Arc;

use axum::http::Request;
use rustboard_api::{
    configuration::get_configuration,
    repository::{comment::PostgresCommentRepository, post::InMemoryPostRepository},
    router::app_routes,
    service::{comment::CommentService, post::PostService},
    state::AppState,
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_without_db() {
    let configuration = Arc::new(get_configuration().expect("Failed to get configuration"));
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database_url)
        .await
        .expect("Failed to connect to database");

    let posts_repo = Arc::new(InMemoryPostRepository::new());
    let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));

    let state = AppState {
        post_service: Arc::new(PostService::new(posts_repo.clone())),
        comment_service: Arc::new(CommentService::new(posts_repo, comments_repo)),
        pool,
        configuration,
    };

    let app = app_routes().with_state(state);
    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
