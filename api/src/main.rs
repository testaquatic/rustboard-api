use rustboard_api::{client::notification::NotificationClient, startup};
use rustboard_domain::{configuration::get_configuration, telemetry};
use sqlx::postgres::PgPoolOptions;

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

    // gRPC 클라이언트 연결
    let notification_client = NotificationClient::connect(&configuration.notification_server_addr)
        .await
        .map_err(|e| {
            tracing::error!("알림 서비스 연결 실패: {}", e);
            e
        })?;

    // TCP 리스너 생성
    let listener = tokio::net::TcpListener::bind(configuration.bind_addr).await?;
    tracing::info!(
        "{} listening on http://{}",
        configuration.service_name,
        listener.local_addr()?
    );

    // 서버 실행
    startup::run_app(
        listener,
        pool.clone(),
        configuration.into(),
        notification_client,
    )
    .await?;

    Ok(())
}
