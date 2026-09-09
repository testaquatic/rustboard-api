use std::{net::SocketAddr, str::FromStr};

use axum::http::StatusCode;
use reqwest::Response;
use rustboard_api::{
    configuration::{DatabaseSettings, Settings},
    domain::post::CreatePostInput,
    startup::run_app,
    telemetry,
};
use serde_json::json;
use sqlx::{QueryBuilder, postgres::PgPoolOptions};
use tokio::{net::TcpListener, task::JoinHandle};
use uuid::Uuid;

pub struct TestContext {
    _join_handle: JoinHandle<Result<(), anyhow::Error>>,
    configuration: Settings,
    reqwest_client: reqwest::Client,
}

impl TestContext {
    pub async fn new() -> Self {
        let mut configuration = get_test_configuration();

        // 로깅
        let _guard = telemetry::init_telemetry(&configuration).expect("텔레메트리 초기화 실패");

        create_test_db(&configuration).await;
        let pool = PgPoolOptions::new()
            .connect(&configuration.database.database_url())
            .await
            .expect("Failed to connect database");

        let listener = TcpListener::bind(configuration.bind_addr)
            .await
            .expect("Failed to bind");

        configuration.bind_addr = SocketAddr::from_str(&format!(
            "127.0.0.1:{}",
            listener.local_addr().unwrap().port()
        ))
        .unwrap();

        let app_info = configuration.clone().into();
        let join_handle = tokio::spawn(async move { run_app(listener, pool, app_info).await });

        let reqwest_clinet = reqwest::Client::new();

        Self {
            _join_handle: join_handle,
            configuration,
            reqwest_client: reqwest_clinet,
        }
    }

    fn url(&self, uri: &str) -> String {
        format!("http://{}{}", self.configuration.bind_addr, uri)
    }

    pub async fn post_json(
        &self,
        uri: &str,
        token: Option<&str>,
        json: &serde_json::Value,
    ) -> Response {
        let mut request = self.reqwest_client.post(self.url(uri)).json(json);

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        request.send().await.expect("POST 요청 실패")
    }

    pub async fn delete(&self, uri: &str, token: Option<&str>) -> Response {
        let mut request = self.reqwest_client.delete(self.url(uri));

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        request.send().await.expect("DELETE 요청 실패")
    }

    pub async fn get(&self, uri: &str, token: Option<&str>) -> (StatusCode, serde_json::Value) {
        let mut request = self.reqwest_client.get(self.url(uri));

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.expect("GET 요청 실패");
        let status_code = response.status();
        let json_body = response.json().await.expect("JSON 파싱 실패");

        (status_code, json_body)
    }

    pub async fn patch_json(
        &self,
        uri: &str,
        token: Option<&str>,
        json: &serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut request = self.reqwest_client.patch(self.url(uri)).json(json);

        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await.expect("PATCH 요청 실패");
        let status_code = response.status();
        let json_body = response.json().await.expect("JSON 파싱 실패");

        (status_code, json_body)
    }

    /// 회원 가입과 로그인을 한 후 토큰을 반환한다.
    pub async fn signup_and_login(&self) -> Option<String> {
        // 회원 가입
        let response = self
            .post_json(
                "/signup",
                None,
                &json!({
                    "email": "test@example.com",
                    "password": "password123",
                    "display_name": "Tester",
                }),
            )
            .await;

        assert_eq!(response.status(), StatusCode::CREATED);

        // 로그인
        let response = self
            .post_json(
                "/login",
                None,
                &json!({
                    "email": "test@example.com",
                    "password": "password123",
                }),
            )
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let json_body = response_json(response).await;
        json_body["token"].as_str().map(String::from)
    }

    /// 글을 주입한다
    pub async fn seed_post(&self, posts: &[CreatePostInput], token: &str) -> Vec<Response> {
        let mut responses = Vec::new();
        for post in posts {
            let response = self
                .post_json(
                    "/posts",
                    Some(token),
                    &serde_json::json!({
                        "title": post.title,
                        "content": post.content,
                    }),
                )
                .await;
            responses.push(response);
        }
        responses
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
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        otel_exporter_otlp_endpoint: "http://localhost:4317".to_string(),
    }
}

async fn create_test_db(configuration: &Settings) {
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
}

pub async fn response_json(response: Response) -> serde_json::Value {
    response.json().await.expect("JSON 파싱 실패")
}

// pub async fn response_text(response: Response) -> String {
//     let body = axum::body::to_bytes(response.into_body(), usize::MAX)
//         .await
//         .unwrap();
//     String::from_utf8(body.to_vec()).unwrap()
// }
