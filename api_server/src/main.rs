use std::{net::SocketAddr, sync::Arc, time::Duration};

use rustboard_api::{
    client::notification::NotificationClient,
    config::Config,
    repository::{
        comment::PostgresCommentRepository, posts::PostgresPostRepository,
        user::PostgresUserRepository,
    },
    router::create_app_router_with_middleware,
    service::{comments::CommentService, posts::PostService, user::UserService},
    shutdown::shutdown_signal,
    state::AppState,
};
use rustboard_proto::notification::notification_service_server::NotificationServiceServer;
use rustboard_telemetry::telemetry;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // env 파일이 있으면 로드한다.
    dotenvy::dotenv().ok();

    // 로그 설정
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "rustboard_api=debug,tower_http=debug,sqlx=info".into()
        } else {
            "rustboard_api=info,tower_http=info,sqlx=warn".into()
        }
    });
    // 우아한 종료를 위한 _gurad
    // 텔레메트를 활성화한다.
    let _guard = telemetry::init_telemetry(env_filter)?;

    // 설정 읽기
    let config = Arc::new(Config::from_env()?);

    // gRPC 알림 서비스
    let notifier = rustboard_notifier::service::NotifierService::new();

    let config_clone = config.clone();
    // gRPC 서버를 먼저 실행
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(NotificationServiceServer::new(notifier))
            .serve_with_shutdown(config_clone.grpc_bind_addr, shutdown_signal())
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "gRPC 서버 오류");
            })
            .expect("gRPC 서버 오류")
    });

    // gRPC 클라이언트 생성
    let notification_client =
        NotificationClient::connect(&format!("http://{}", (&config.grpc_server_addr)))
            .await
            .expect("gRPC 알림 서버 연결 실패");

    // DB 연결 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // 앱을 시작하면 DB마이그레이션을 자동 적용한다.
    sqlx::migrate!("../migrations").run(&pool).await?;

    // 리포지토리 초기화
    let posts_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));

    // 서비스 초기화
    let posts_service = Arc::new(PostService::new(posts_repo.clone()));
    let comments_service = Arc::new(CommentService::new(
        posts_repo,
        comments_repo.clone(),
        notification_client.clone(),
    ));
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let users_service = Arc::new(UserService::new(user_repo));

    // 동시 접근수를 제한하는 세마포어
    let ws_semaphore = Arc::new(tokio::sync::Semaphore::new(config.grpc_max_connections));

    // AppState 생성
    let state = AppState {
        config: config.clone(),
        posts_service,
        comments_service,
        users_service,
        ws_semaphore,
        notification_client,
    };

    // 라우터를 생성하고 상태와 미들웨어를 붙인다
    let app = create_app_router_with_middleware(&config, state);
    // 서버 시작
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;

    tracing::info!("HTTP 서버: {}", config.bind_addr);
    tracing::info!("gRPC 서버: {}", config.grpc_server_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "HTTP 서버 오류");
    })
    .expect("HTTP 서버 오류");

    // 서버 종료 후 리소스 정리
    tracing::info!("리소스 정리를 시작합니다 (최대 10초)");

    tokio::select! {
        _ = cleanup(pool.clone()) => tracing::info!("리소스 정리 완료"),
        _ = tokio::time::sleep(Duration::from_secs(10)) => tracing::warn!("리소스 정리 타입아웃, 강제 종료합니다"),
    }

    tracing::info!("서버 종료 완료");

    Ok(())
}

async fn cleanup(pool: sqlx::PgPool) {
    // DB 커넥션 풀 정리
    pool.close().await;
    tracing::info!("DB 커넥션 풀 정리 완료");
}
