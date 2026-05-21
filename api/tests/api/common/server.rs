use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};

use axum::Router;

use reqwest::header;
use rustboard_api::{
    client::notification::NotificationClient,
    config::Config,
    repository::{
        comment::InMemoryCommentRepository, posts::InMemoryPostRepository,
        user::InMemoryUserRepository,
    },
    router::create_app_router,
    service::{comments::CommentService, posts::PostService, user::UserService},
    state::AppState,
};
use rustboard_domain::posts::Post;
use rustboard_proto::notification::notification_service_server::NotificationServiceServer;
use rustboard_telemetry::telemetry::{OtelGuard, init_telemetry};
use serde_json::json;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;
use tokio::{net::TcpListener, task::JoinHandle};
use tracing_subscriber::EnvFilter;

/// 실제로 작동하는 서버이다.
pub struct TestServer {
    pub state: AppState,
    pub app_router: Router,
    _server_handle: JoinHandle<()>,
    _grpc_server_handle: JoinHandle<()>,
    _postgres_container: Option<ContainerAsync<Postgres>>,
    reqwest_client: reqwest::Client,
}

impl TestServer {
    pub async fn new_in_memory() -> TestServer {
        // 한번만 실행되도록 LazyLock을 사용한다.
        static _INIT_ONCE: LazyLock<OtelGuard> = LazyLock::new(|| {
            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustboard_api=debug,tower_http=debug,sqlx=info".into());
            init_telemetry(env_filter).unwrap()
        });

        let posts_repo = Arc::new(InMemoryPostRepository::new());
        let comments_repo = Arc::new(InMemoryCommentRepository::new(posts_repo.clone()));
        let (gprc_server, notification_client, grpc_socket_addr) = spawn_grpc_server().await;

        let posts_service = Arc::new(PostService::new(posts_repo.clone()));
        let comments_service = Arc::new(CommentService::new(
            posts_repo,
            comments_repo,
            notification_client.clone(),
        ));

        let users_service = Arc::new(UserService::new(Arc::new(InMemoryUserRepository::new())));
        let ws_semaphore = Arc::new(tokio::sync::Semaphore::new(100));

        let api_server_listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");

        let config = Config {
            bind_addr: api_server_listener
                .local_addr()
                .expect("Failed to get local address"),
            grpc_bind_addr: grpc_socket_addr,
            grpc_server_addr: grpc_socket_addr,
            grpc_max_connections: 1000,
            service_name: format!("rustboard-api-test-{}", uuid::Uuid::new_v4()),
            database_url: "".to_string(),
            jwt_secret: "test-secret-key-for-testing-only".into(),
            jwt_expiration_minutes: 15,
        };

        let app_state = AppState {
            posts_service,
            comments_service,
            users_service,
            ws_semaphore,
            notification_client,
            config: Arc::new(config.clone()),
        };

        let app_router = create_app_router(&config, app_state.clone());
        let app_router_clone = app_router.clone();

        let server_handle = tokio::spawn(async move {
            axum::serve(api_server_listener, app_router_clone)
                .await
                .expect("test api server error")
        });

        let reqwest_client = reqwest::Client::new();
        TestServer {
            state: app_state,
            app_router,
            _server_handle: server_handle,
            _grpc_server_handle: gprc_server,
            _postgres_container: None,
            reqwest_client,
        }
    }

    // /// 테스트 서버 인스턴스를 생성한다.
    // pub async fn new() -> TestServer {
    //     // 한번만 실행되도록 LazyLock을 사용한다.
    //     static _INIT_ONCE: LazyLock<OtelGuard> = LazyLock::new(|| {
    //         let env_filter = EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "rustboard_api=debug,tower_http=debug,sqlx=info".into());
    //         init_telemetry(env_filter).unwrap()
    //     });

    //     let postgres_contaienr = Postgres::default()
    //         .start()
    //         .await
    //         .expect("Failed to start postgres container");

    //     let host_port = postgres_contaienr
    //         .get_host_port_ipv4(5432)
    //         .await
    //         .expect("포트 매핑 실패");

    //     let database_url = format!(
    //         "postgres://postgres:postgres@127.0.0.1:{}/postgres",
    //         host_port
    //     );

    //     let pool = PgPool::connect(&database_url).await.expect("DB 연결 실패");
    //     let posts_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    //     let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));
    //     let (gprc_server, notification_client, grpc_socket_addr) = spawn_grpc_server().await;

    //     let posts_service = Arc::new(PostService::new(posts_repo.clone()));
    //     let comments_service = Arc::new(CommentService::new(
    //         posts_repo,
    //         comments_repo,
    //         notification_client.clone(),
    //     ));

    //     let users_service = Arc::new(UserService::new(Arc::new(PostgresUserRepository::new(
    //         pool.clone(),
    //     ))));
    //     let ws_semaphore = Arc::new(tokio::sync::Semaphore::new(100));

    //     let api_server_listener = TcpListener::bind("127.0.0.1:0")
    //         .await
    //         .expect("Failed to bind to random port");

    //     let config = Config {
    //         bind_addr: api_server_listener
    //             .local_addr()
    //             .expect("Failed to get local address"),
    //         grpc_server_addr: grpc_socket_addr,
    //         grpc_bind_addr: grpc_socket_addr,
    //         grpc_max_connections: 1000,
    //         service_name: format!("rustboard-api-test-{}", uuid::Uuid::new_v4()),
    //         database_url,
    //         jwt_secret: "test-secret-key-for-testing-only".into(),
    //         jwt_expiration_minutes: 15,
    //     };

    //     let app_state = AppState {
    //         posts_service,
    //         comments_service,
    //         users_service,
    //         ws_semaphore,
    //         notification_client,
    //         config: Arc::new(config.clone()),
    //     };

    //     let app_router = create_app_router_with_middleware(&config, app_state.clone());
    //     let app_router_clone = app_router.clone();

    //     let server_handle = tokio::spawn(async move {
    //         axum::serve(api_server_listener, app_router_clone)
    //             .await
    //             .expect("test api server error")
    //     });

    //     let reqwest_client = reqwest::Client::new();

    //     TestServer {
    //         state: app_state,
    //         app_router,
    //         _server_handle: server_handle,
    //         _grpc_server_handle: gprc_server,
    //         _postgres_container: Some(postgres_contaienr),
    //         reqwest_client,
    //     }
    // }

    /// 회원가입과 로그인, 토큰 생성을 한번에 수행한다.
    /// 문서와 다르게 reqwest를 사용해서 직접 요청을 생성한다.
    /// 테스트와 실제 환경을 최대한 일치시키고 싶어서이다.
    pub async fn create_test_token(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> String {
        // 회원가입
        let signup_body = serde_json::json!({
            "email": email,
            "password": password,
            "display_name": display_name,
        });

        let response = self
            .reqwest_client
            .post(format!("http://{}/signup", self.state.config.bind_addr))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&signup_body)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::CREATED);

        // 로그인
        let login_body = json!({"email": email, "password": password});
        let token_response = self
            .reqwest_client
            .post(format!("http://{}/login", self.state.config.bind_addr))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&login_body)
            .send()
            .await
            .unwrap();
        assert_eq!(token_response.status(), reqwest::StatusCode::OK);

        let token = token_response.json::<serde_json::Value>().await.unwrap()["token"]
            .as_str()
            .unwrap()
            .to_string();

        token
    }
    /// 글을 작성한다.
    pub async fn create_post(&self, token: &str, post: &Post) {
        let post_body = json!({
            "title": post.title,
            "content": post.body,
        });

        let response = self
            .reqwest_client
            .post(format!("http://{}/posts", self.state.config.bind_addr))
            .bearer_auth(token)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&post_body)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    }

    // /// 정해준 수의 임의의 글을 생성한다
    // pub async fn create_test_post(&self, token: &str, count: usize) {
    //     for _ in 0..count {
    //         let title = Sentence(5..20).fake::<String>();
    //         let content = Paragraph(5..20).fake::<String>();
    //         let post_body = json!({
    //             "title": title,
    //             "content": content,
    //         });

    //         let response = self
    //             .reqwest_client
    //             .post(format!("http://{}/posts", self.state.config.bind_addr))
    //             .bearer_auth(token)
    //             .header(header::CONTENT_TYPE, "application/json")
    //             .json(&post_body)
    //             .send()
    //             .await
    //             .unwrap();
    //         assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    //     }
    // }

    // /// 임의의 댓글을 작성한다.

    // pub async fn create_test_comment(&self, token: &str, post_id: u64) {
    //     let comment_body = json!({
    //         "body": Sentence(6..20).fake::<String>(),
    //     });

    //     self.reqwest_client
    //         .post(format!(
    //             "http://{}/posts/{}/comments",
    //             self.state.config.bind_addr, post_id
    //         ))
    //         .bearer_auth(token)
    //         .header(header::CONTENT_TYPE, "application/json")
    //         .json(&comment_body)
    //         .send()
    //         .await
    //         .unwrap();
    // }
}

// pub fn build_ws_reqeust(addr: &str, token: &str) -> Request<()> {
//     Request::builder()
//         .uri(format!("ws://{}/ws/notifications", addr))
//         .header(header::AUTHORIZATION, format!("Bearer {}", token))
//         .header(header::HOST, addr)
//         .header(header::CONNECTION, "Upgrade")
//         .header(header::UPGRADE, "websocket")
//         .header(header::SEC_WEBSOCKET_VERSION, "13")
//         .header(
//             header::SEC_WEBSOCKET_KEY,
//             tokio_tungstenite::tungstenite::handshake::client::generate_key(),
//         )
//         .body(())
//         .unwrap()
// }

/// gRPC 서버를 테스트용으로 생성하고 백그라운드에서 실행
pub async fn spawn_grpc_server() -> (tokio::task::JoinHandle<()>, NotificationClient, SocketAddr) {
    // 테스트용 gRPC 서버를 백그라운드에서 실행
    let notifier = rustboard_notifier::service::NotifierService::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound_addr = listener.local_addr().unwrap();

    let grpc_server = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(NotificationServiceServer::new(notifier))
            .serve(bound_addr)
            .await
            .unwrap()
    });

    let client = NotificationClient::connect(&format!("http://{}", bound_addr))
        .await
        .unwrap();

    (grpc_server, client, bound_addr)
}
