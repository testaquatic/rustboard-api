use std::net::SocketAddr;

use rustboard_api::client::notification::NotificationClient;
use rustboard_proto::notification::notification_service_server::NotificationServiceServer;

#[tokio::test]
async fn test_notification_client() {
    // 테스트용 gRPC 서비를 백그라운드에 적용
    let notifier = rustboard_notifier::service::NotifierService::new();
    let addr = "127.0.0.1:50051".parse::<SocketAddr>().unwrap();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let bound_addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(NotificationServiceServer::new(notifier))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // 클라이언트 연결
    let client = NotificationClient::connect(&format!("http://{}", bound_addr))
        .await
        .unwrap();

    // 호출
    let response = client
        .send_comment_notificaiton("user-123", "alice", 42, 99)
        .await
        .unwrap();

    assert!(response.success);
    assert!(!response.message_id.is_empty());
}
