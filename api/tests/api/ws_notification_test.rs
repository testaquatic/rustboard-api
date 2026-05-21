use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::common::server::{TestServer, build_ws_reqeust};

/// 여기부터는 문서의 테스트이다.
#[tokio::test]
async fn test_comment_notification() {
    let test_server = TestServer::new().await;

    // 토큰 생성
    let test_user_token = test_server
        .create_test_token("test@example.com", "password123", "tester")
        .await;

    // ws 연결
    let request = build_ws_reqeust(
        &test_server.state.config.grpc_addr.to_string(),
        &test_user_token,
    );

    let (mut ws, _) = connect_async(request).await.unwrap();

    // 게시글 작성
    test_server.create_test_post(&test_user_token, 10).await;

    // 9번 게시글 구독
    ws.send(Message::Text(
        json!({"action": "subscribe", "post_id": 9})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();

    // 잠시 대기
    time::sleep(Duration::from_millis(100)).await;

    let other_user_token = test_server
        .create_test_token("other@example.com", "password123", "other")
        .await;

    // 댓글 생성
    test_server.create_test_comment(&other_user_token, 9).await;

    // 알림 수신
    let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("Timeout")
        .unwrap()
        .unwrap();

    if let Message::Text(text) = msg {
        let notification: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(notification["post_id"], 9);
        assert_eq!(notification["type"], "comment_added");
        assert_eq!(notification["actor"], "other");
    } else {
        panic!("텍스트 메시지가 아님: {:?}", msg);
    }
}

#[tokio::test]
async fn test_unsubscribed_post_filtered() {
    let test_server = TestServer::new().await;
    let token = test_server
        .create_test_token("test@example.com", "password123", "tester")
        .await;
    let request = build_ws_reqeust(&test_server.state.config.grpc_addr.to_string(), &token);

    let (mut ws, _) = connect_async(request).await.unwrap();
    // 글을 10개 작성함
    test_server.create_test_post(&token, 10).await;

    // 9번 글 알림 신청
    ws.send(Message::Text(
        serde_json::json!({"action": "subscribe", "post_id": 9})
            .to_string()
            .into(),
    ))
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let someone_token = test_server
        .create_test_token("someone@example.com", "password123", "someone")
        .await;

    // 10번 글에 댓글
    test_server.create_test_comment(&someone_token, 10).await;

    let result = tokio::time::timeout(Duration::from_secs(3), ws.next()).await;

    // 알림이 오지 않아야 함
    assert!(result.is_err(), "구독하지 않은 게시글의 알림이 도착함");

    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn test_ws_without_auth_rejection() {
    let test_server = TestServer::new().await;
    let result = connect_async(format!(
        "ws://{}/ws/notifications",
        test_server.state.config.grpc_addr
    ))
    .await;

    assert!(result.is_err(), "인증 없이 연결 성공함");
}
