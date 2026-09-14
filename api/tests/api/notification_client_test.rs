use rustboard_api::client::notification::NotificationClient;

use crate::common::test_context::TestContext;

#[tokio::test]
async fn test_send_comment_notification() {
    let test_context = TestContext::new().await;

    // 클라이언트 연결
    let client = NotificationClient::connect(&test_context.configuration.notification_server_addr)
        .await
        .expect("NotificationClient 연결 실패");

    let response = client
        .send_comment_notification("user-123", "alice", 42, 99)
        .await
        .expect("알림 전송 실패");

    assert!(response.success);
    assert!(!response.message_id.is_empty());
}
