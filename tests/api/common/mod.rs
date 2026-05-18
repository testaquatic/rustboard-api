pub mod server;

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{self, Request, header},
};
use http_body_util::BodyExt;
use rustboard_api::{
    config::Config,
    domain::posts::Post,
    repository::{
        comment::{DynCommentRepository, InMemoryCommentRepository, PostgresCommentRepository},
        posts::{DynPostRepository, InMemoryPostRepository, PostgresPostRepository},
        user::{DynUserRepository, InMemoryUserRepository, PostgresUserRepository},
    },
    router,
    service::{comments::CommentService, posts::PostService, user::UserService},
    state::AppState,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;

pub struct TestContext {
    pub post_repo: Arc<InMemoryPostRepository>,
    _comment_repo: Arc<InMemoryCommentRepository>,
    _user_repo: Arc<InMemoryUserRepository>,
    config: Arc<Config>,
    state: AppState,
}

impl TestContext {
    pub fn new_in_memory() -> Self {
        let post_repo = Arc::new(InMemoryPostRepository::new());
        let _comment_repo = Arc::new(InMemoryCommentRepository::new());
        let _user_repo = Arc::new(InMemoryUserRepository::new());
        let config = Arc::new(Config::test_default());
        let state =
            AppState::new_for_test(post_repo.clone(), _comment_repo.clone(), _user_repo.clone());

        Self {
            post_repo,
            _comment_repo,
            _user_repo,
            config,
            state,
        }
    }

    pub fn app(&self) -> Router {
        router::create_router(&self.config, self.state.clone())
    }

    /// 계정을 생성하고, 로그인한 후에 토큰을 반환한다.
    pub async fn signup_and_login(&self) -> String {
        signup_and_login(|| self.app()).await
    }
}

/// 테스트용 함수는 되도록 노출하지 않는 것이 좋다고 해서 별도로 정리했다.

pub trait ConfigTestExt {
    fn test_default() -> Self;
}

impl ConfigTestExt for Config {
    fn test_default() -> Self {
        Self {
            bind_addr: "127.0.0.1:3000"
                .parse()
                .expect("Cannot parse to SocketAddr"),
            service_name: "rustboard-api-test".to_string(),
            // 테스트 할 때는 사용 안 함
            database_url: String::new(),
            jwt_secret: "test-secret-key-for-testing-only".into(),
            jwt_expiration_minutes: 15,
        }
    }
}

pub trait AppStateTestExt {
    fn new_for_test(
        post_repo: DynPostRepository,
        comment_repo: DynCommentRepository,
        user_repo: DynUserRepository,
    ) -> Self;
}

impl AppStateTestExt for AppState {
    fn new_for_test(
        post_repo: DynPostRepository,
        comment_repo: DynCommentRepository,
        user_repo: DynUserRepository,
    ) -> Self {
        let (notify_tx, _) = tokio::sync::broadcast::channel(100);

        let config = Arc::new(Config::test_default());
        let posts_service = Arc::new(PostService::new(post_repo.clone()));
        let comments_service = Arc::new(CommentService::new(
            post_repo,
            comment_repo,
            notify_tx.clone(),
        ));
        let users_service = Arc::new(UserService::new(user_repo));

        let ws_semaphore = Arc::new(tokio::sync::Semaphore::new(100));

        Self {
            config,
            posts_service,
            comments_service,
            users_service,
            notify_tx,
            ws_semaphore,
        }
    }
}

pub trait InMemoryPostRepositoryTestExt {
    async fn seed(&self, posts: Vec<Post>);
}

impl InMemoryPostRepositoryTestExt for InMemoryPostRepository {
    async fn seed(&self, posts: Vec<Post>) {
        let mut state = self.lock().await;
        posts.into_iter().for_each(|post| {
            let id = post.id;
            state.items.insert(id, post);
            state.next_id = std::cmp::max(state.next_id, id + 1);
        });
    }
}

pub struct IntegrationTestContext {
    pub pool: PgPool,
    _container: ContainerAsync<Postgres>,
    pub database_url: String,
}

impl IntegrationTestContext {
    pub async fn new() -> Self {
        // 컨테이너 시작
        let _container = Postgres::default()
            .start()
            .await
            .expect("PostgresSQL 컨테이너 시작 실패");

        let host_port = _container
            .get_host_port_ipv4(5432)
            .await
            .expect("포트 매핑 실패");

        let database_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        // 연결 풀 생성
        let pool = PgPool::connect(&database_url).await.expect("DB 연결 실패");

        // 마이그레이션 실행
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("마이그레이션 실패");

        Self {
            pool,
            _container,
            database_url,
        }
    }

    pub fn app_with_db(&self) -> (Router, AppState) {
        let config = Arc::new(Config::test_default());
        let post_repo = Arc::new(PostgresPostRepository::new(self.pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(self.pool.clone()));
        let comment_repo = Arc::new(PostgresCommentRepository::new(self.pool.clone()));

        let (notify_tx, _) = tokio::sync::broadcast::channel(100);

        let posts_service = Arc::new(PostService::new(post_repo.clone()));
        let users_service = Arc::new(UserService::new(user_repo.clone()));
        let comments_service = Arc::new(CommentService::new(
            post_repo,
            comment_repo,
            notify_tx.clone(),
        ));
        let ws_semaphore = Arc::new(tokio::sync::Semaphore::new(100));

        let state = AppState {
            config: config.clone(),
            posts_service,
            comments_service,
            users_service,
            notify_tx,
            ws_semaphore,
        };
        let router = router::create_router(&config, state.clone());

        (router, state)
    }
}

pub async fn signup_and_login(app_fn: impl Fn() -> Router) -> String {
    // 회원가입
    let signup_req = Request::builder()
        .method(http::Method::POST)
        .uri("/signup")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester",
            })
            .to_string(),
        ))
        .unwrap();

    let response = app_fn().oneshot(signup_req).await.unwrap();
    assert_eq!(response.status(), http::StatusCode::CREATED);

    // 로그인
    let login_req = Request::builder()
        .method(http::Method::POST)
        .uri("/login")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com",
                "password": "password123",
            })
            .to_string(),
        ))
        .unwrap();

    let response = app_fn().oneshot(login_req).await.unwrap();
    assert_eq!(response.status(), http::StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice::<'_, Value>(&body).unwrap();
    json["token"].as_str().unwrap().to_string()
}

pub fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

pub fn post_json(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(http::Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub fn patch_json(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(http::Method::PATCH)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub fn delete(uri: &str) -> Request<Body> {
    Request::builder()
        .method(http::Method::DELETE)
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

pub async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}
