use rustboard_api::{client::notification::NotificationClient, shutdown::shutdown_signal, startup};
use rustboard_domain::{configuration::get_configuration, telemetry};
use rustboard_proto::notification::notification_service_server::NotificationServiceServer;
use sqlx::postgres::PgPoolOptions;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 설정을 읽는다
    let configuration = get_configuration()?;

    // 로깅
    let _guard = telemetry::init_telemetry(&configuration)?;

    // DB 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.database_url())
        .await
        .map_err(|e| {
            tracing::error!("DB 연결 실패: {}", e);
            e
        })?;

    // gRPC 알림 서비스
    let notifier = rustboard_notifier::service::NotifierSevice::new();
    let grpc_server_addr = configuration.notification_server_addr.clone();
    let grpc_server_addr = grpc_server_addr
        .strip_prefix("http://")
        .or(grpc_server_addr.strip_prefix("https://"))
        .expect("gRPC 서버 주소 파싱 실패")
        .parse()
        .expect("gRPC 서버 주소 파싱 실패");
    tracing::info!("gRPC 서버: {}", grpc_server_addr);
    let grpc_handle = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(NotificationServiceServer::new(notifier))
            .serve_with_shutdown(grpc_server_addr, shutdown_signal())
            .await
    });

    // gRPC 클라이언트 연결
    // gRPC 서버 실행 대기
    let mut remain_retries = 3;
    let notification_client = loop {
        match NotificationClient::connect(&configuration.notification_server_addr).await {
            Err(e) => {
                tracing::error!(
                    "알림 서비스 연결 실패: {}, 남은 재시도 수: {}",
                    e,
                    remain_retries
                );
                if remain_retries == 0 {
                    return Err(e.into());
                }
                remain_retries -= 1;
                time::sleep(time::Duration::from_secs(1)).await;
            }
            Ok(client) => {
                tracing::info!("알림 서비스 연결 성공");
                break client;
            }
        }
    };

    // TCP 리스너 생성
    let listener = tokio::net::TcpListener::bind(configuration.bind_addr).await?;
    tracing::info!("HTTP 서버: {}", listener.local_addr()?);

    // 두 서버를 동시에 실행
    tokio::select! {
            http_result = startup::run_app(
                listener,
                pool.clone(),
                configuration.into(),
                notification_client) => {
                    if let Err(e) = http_result {
                        tracing::error!(error = %e, "HTTP 서버 에러");
                    }
                }
            gprc_result = grpc_handle => {
                if let Err(e) = gprc_result {
                    tracing::error!(error = %e, "gRPC 서버 에러");
                }
            }

    };

    Ok(())
}
