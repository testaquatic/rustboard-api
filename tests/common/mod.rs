use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use rustboard_api::{
    configuration::{Settings, get_configuration},
    repository::{
        comment::PostgresCommentRepository, post::PostgresPostRepository,
        user::PostgresUserRepository,
    },
    service::{comment::CommentService, post::PostService, user::UserService},
    state::AppState,
};
use serde_json::{Value, json};
use sqlx::{QueryBuilder, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

pub async fn test_app() -> Router {
    let configuration = create_test_db().await;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.database_url())
        .await
        .expect("Failed to connect database");

    let posts_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));
    let users_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

    let post_service = Arc::new(PostService::new(posts_repo.clone()));
    let comment_service = Arc::new(CommentService::new(posts_repo, comments_repo));
    let user_service = Arc::new(UserService::new(users_repo));

    let state = AppState {
        post_service,
        comment_service,
        configuration: Arc::new(configuration),
        pool,
        user_service,
    };

    rustboard_api::router::create_router(state)
}

async fn create_test_db() -> Settings {
    let mut configuration = get_configuration().unwrap();
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "postgres://{}:{}@{}:{}",
            configuration.database.username,
            configuration.database.password,
            configuration.database.host,
            configuration.database.port,
        ))
        .await
        .expect("Failed to connect database");

    let mut query = QueryBuilder::new(format!(
        r#"CREATE DATABASE "{}";"#,
        configuration.database.database_name
    ));
    query.build().execute(&pool).await.expect("DB 생성 실패");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.database_url())
        .await
        .expect("Failed to connect database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("DB 마이그레이션 실패");

    configuration
}

pub async fn signup_and_login(app_fn: impl Fn() -> Router) -> String {
    // 회원가입
    let signup_req = Request::builder()
        .uri("/signup")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester",
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app_fn().oneshot(signup_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 로그인
    let login_req = Request::builder()
        .uri("/login")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "email": "test@example.com",
                "password": "password123",
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app_fn().oneshot(login_req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response = serde_json::from_slice::<Value>(&body).unwrap();

    response["access_token"].as_str().unwrap().to_string()
}
