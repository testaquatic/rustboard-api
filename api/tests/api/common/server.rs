use std::{net::SocketAddr, sync::LazyLock};

use fake::{
    Fake,
    faker::lorem::en::{Paragraph, Sentence},
};
use reqwest::header;
use rustboard_api::{config::Config, router::create_app_router_with_middleware, state::AppState};
use rustboard_telemetry::telemetry::{OtelGuard, init_telemetry};
use serde_json::json;
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_tungstenite::tungstenite::http::Request;

use crate::common::{ConfigTestExt, IntegrationTestContext};

/// 실제로 작동하는 서버이다.
pub struct TestServer {
    pub addr: String,
    reqwest_client: reqwest::Client,
    _state: AppState,
    _postgres_context: IntegrationTestContext,
    _handle: JoinHandle<()>,
}

impl TestServer {
    /// 테스트 서버 인스턴스를 생성한다.
    pub async fn new() -> TestServer {
        // 한번만 실행되도록 LazyLock을 사용한다.
        static _INIT_ONCE: LazyLock<OtelGuard> = LazyLock::new(|| init_telemetry().unwrap());

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");

        let postgres_context = IntegrationTestContext::new().await;
        let (_, state) = postgres_context.app_with_db();
        let mut config = Config::test_default();
        config.bind_addr = listener.local_addr().expect("Failed to get local address");
        config.database_url = postgres_context.database_url.clone();

        let addr = config.bind_addr.to_string();

        let router = create_app_router_with_middleware(&config, state.clone());

        let server = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        );

        let handle = tokio::spawn(async move {
            if let Err(e) = server.await {
                tracing::error!(error = %e, "서버 오류");
            }
        });

        let reqwest_client = reqwest::Client::new();

        TestServer {
            addr,
            reqwest_client,
            _state: state,
            _postgres_context: postgres_context,
            _handle: handle,
        }
    }

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
            .post(format!("http://{}/signup", self.addr))
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
            .post(format!("http://{}/login", self.addr))
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

    /// 정해준 수의 임의의 글을 생성한다
    pub async fn create_test_post(&self, token: &str, count: usize) {
        for _ in 0..count {
            let title = Sentence(5..20).fake::<String>();
            let content = Paragraph(5..20).fake::<String>();
            let post_body = json!({
                "title": title,
                "content": content,
            });

            let response = self
                .reqwest_client
                .post(format!("http://{}/posts", self.addr))
                .bearer_auth(token)
                .header(header::CONTENT_TYPE, "application/json")
                .json(&post_body)
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), reqwest::StatusCode::CREATED);
        }
    }

    /// 임의의 댓글을 작성한다.
    pub async fn create_test_comment(&self, token: &str, post_id: u64) {
        let comment_body = json!({
            "body": Sentence(6..20).fake::<String>(),
        });

        self.reqwest_client
            .post(format!("http://{}/posts/{}/comments", self.addr, post_id))
            .bearer_auth(token)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&comment_body)
            .send()
            .await
            .unwrap();
    }
}

pub fn build_ws_reqeust(addr: &str, token: &str) -> Request<()> {
    Request::builder()
        .uri(format!("ws://{}/ws/notifications", addr))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::HOST, addr)
        .header(header::CONNECTION, "Upgrade")
        .header(header::UPGRADE, "websocket")
        .header(header::SEC_WEBSOCKET_VERSION, "13")
        .header(
            header::SEC_WEBSOCKET_KEY,
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .body(())
        .unwrap()
}
