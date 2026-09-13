use rustboard_notifier::service::NotifierSevice;
use rustboard_proto::notification::notification_service_server::NotificationServiceServer;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(
        if cfg!(debug_assertions) {
            "rustboard_notifier=debug"
        } else {
            "rustboard_notifier=info"
        }
        .into(),
    );

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let addr = "0.0.0.0:50051";
    let notifier = NotifierSevice::new();

    tracing::info!(%addr, "gRPC 서버 시작");

    Server::builder()
        .add_service(NotificationServiceServer::new(notifier))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
