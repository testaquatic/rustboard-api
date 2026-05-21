pub async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("SIGTERM 시그널 핸들러 설치 실패");

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {tracing::info!("Ctrl+C 수신, 종료 시작")},
        _ = sigterm.recv() => {tracing::info!("SIGTERM 수신, 종료 시작")},
    }
}
