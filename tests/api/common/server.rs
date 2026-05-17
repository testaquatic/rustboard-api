use std::net::SocketAddr;

use rustboard_api::{
    config::Config,
    router::create_app_router_with_middleware,
    state::AppState,
    telemetry::{OtelGuard, init_telemetry},
};
use tokio::{net::TcpListener, task::JoinHandle};

use crate::common::{ConfigTestExt, IntegrationTestContext};

/// 사용 환경과 유사한 서버를 구동한다.
pub struct TestServer {
    pub addr: String,
    _state: AppState,
    _postgres_context: IntegrationTestContext,
    _handle: JoinHandle<()>,
    _telemetry: OtelGuard,
}

impl TestServer {
    pub async fn new() -> TestServer {
        let _telemetry = init_telemetry().unwrap();

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

        TestServer {
            addr,
            _state: state,
            _postgres_context: postgres_context,
            _handle: handle,
            _telemetry,
        }
    }
}
