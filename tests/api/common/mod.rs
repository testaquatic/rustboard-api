use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use rustboard_api::{
    configuration::{DatabaseSettings, Settings, get_configuration},
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

pub struct TestContext {
    post_repo: Arc<PostgresPostRepository>,
    user_repo: Arc<PostgresUserRepository>,
    comment_repo: Arc<PostgresCommentRepository>,
    state: AppState,
}

impl TestContext {
    pub async fn new() -> Self {
        let configuration = create_test_db().await;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&configuration.database.database_url())
            .await
            .expect("Failed to connect database");

        let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
        let comment_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

        let post_service = Arc::new(PostService::new(post_repo.clone()));
        let comment_service =
            Arc::new(CommentService::new(post_repo.clone(), comment_repo.clone()));
        let user_service = Arc::new(UserService::new(user_repo.clone()));

        let state = AppState {
            post_service,
            comment_service,
            configuration: Arc::new(configuration),
            pool,
            user_service,
        };

        Self {
            post_repo,
            user_repo,
            comment_repo,
            state,
        }
    }

    pub fn app(&self) -> Router {
        rustboard_api::router::create_router(self.state.clone())
    }
}

fn get_test_configuration() -> Settings {
    Settings {
        database: DatabaseSettings {
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database_name: Uuid::new_v4().to_string(),
            host: "localhost".to_string(),
            port: 5432,
        },
        jwt_secret: "test-secret-key-for-testing-only".to_string(),
        jwt_expiration_minutes: 15,
        service_name: "rustboard-api-test".to_string(),
        bind_addr: "127.0.0.1:3000".parse().unwrap(),
    }
}

async fn create_test_db() -> Settings {
    let configuration = get_test_configuration();
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

pub fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

pub fn post_json(uri: &str, body: &serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(body).unwrap()))
        .unwrap()
}

pub fn patch_json(uri: &str, body: &serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(Method::PATCH)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(body).unwrap()))
        .unwrap()
}

pub fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

pub fn with_token(mut request: Request<Body>, token: &str) -> Request<Body> {
    request.headers_mut().insert(
        header::AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );

    request
}

pub async fn response_json(response: Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub async fn response_text(response: Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}
