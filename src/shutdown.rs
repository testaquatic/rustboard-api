use tokio::signal;

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C 핸들러 설치 실패");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("SIGTERM 핸들러 설치 실패")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl+C 수신, 종료를 시작합니다")
        }
        _ = terminate => {
            tracing::info!("SIGTERM 수신, 종료를 시작합니다")
        }
    }
}
