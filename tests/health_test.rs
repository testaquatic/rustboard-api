use std::sync::Arc;

use axum::{Router, http::Request, routing::get};
use rustboard_api::{
    configuration::get_configuration,
    repository::{
        comment::PostgresCommentRepository, post::PostgresPostRepository,
        user::PostgresUserRepository,
    },
    routes::meta::health,
    service::{comment::CommentService, post::PostService, user::UserService},
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

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let posts_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));
    let users_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let state = AppState {
        post_service: Arc::new(PostService::new(posts_repo.clone())),
        comment_service: Arc::new(CommentService::new(posts_repo, comments_repo)),
        user_service: Arc::new(UserService::new(users_repo)),
        pool,
        configuration,
    };

    let app = Router::new()
        .route("/health", get(health))
        .with_state(state);
    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
