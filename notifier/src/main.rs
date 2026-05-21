use rustboard_notifier::service::NotifierService;
use rustboard_proto::notification::notification_service_server::NotificationServiceServer;
use rustboard_telemetry::telemetry::init_telemetry;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "rustboard_notifier=debug,tower_http=debug,sqlx=info".into()
        } else {
            "rustboard_notifier=info,tower_http=info,sqlx=warn".into()
        }
    });
    let _otel_guard = init_telemetry(env_filter)?;

    let addr = "0.0.0.0:50051".parse()?;
    let notifier = NotifierService::new();

    tracing::info!(%addr, "gRPC 서버 시작");

    Server::builder()
        .add_service(NotificationServiceServer::new(notifier))
        .serve(addr)
        .await?;

    Ok(())
}
