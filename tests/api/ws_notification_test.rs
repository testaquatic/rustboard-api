use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rustboard_api::domain::{
    notification::{ClientMessage, Notification},
    post::CreatePostInput,
};

use super::common;

#[tokio::test]
async fn test_websocket_notifications() {
    let ctx = common::TestContext::new().await;
    // 토큰 발급
    let token = ctx.signup_and_login().await.unwrap();

    let (mut write, mut read) = ctx.connect_ws(&token, "/ws/notifications").await;

    // 게시글 구독
    write
        .send(tungstenite::Message::Text(
            serde_json::to_string(&ClientMessage::Subscribe { post_id: 10 })
                .unwrap()
                .into(),
        ))
        .await
        .expect("게시글 구독 실패");

    // 글 생성
    let posts = (1..=20)
        .map(|i| CreatePostInput {
            title: format!("{}번째 글", i),
            content: "테스트 글".to_string(),
        })
        .collect::<Vec<_>>();
    ctx.seed_post(&posts, &token).await;

    // 댓글 생성
    ctx.post_json(
        "/posts/10/comments",
        Some(&token),
        &serde_json::json!({
            "body": "댓글입니다",
        }),
    )
    .await;

    // 알림 수신
    let notification = read.next().await.unwrap().unwrap();

    let notification_data =
        serde_json::from_str::<Notification>(&notification.to_string()).unwrap();

    assert_eq!(notification_data.post_id, 10);
    assert!(
        notification_data
            .message
            .contains("10번 게시글에 댓글을 달았습니다"),
        "알림 메시지가 다릅니다: {}",
        notification_data.message
    );

    // 댓글 생성(알림이 오지 않아야 함)
    ctx.post_json(
        "/posts/20/comments",
        Some(&token),
        &serde_json::json!({
            "body": "댓글입니다",
        }),
    )
    .await;

    // 알림 없음 확인
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(200)) => (),
        result = read.next() => {
            panic!("알림이 오면 안됨: {:?}", result);
        }
    }

    // 구독을 해제한다.
    write
        .send(
            serde_json::to_string(&ClientMessage::Unsubscribe { post_id: 10 })
                .unwrap()
                .into(),
        )
        .await
        .expect("구독 해제 실패");

    // 댓글 생성
    ctx.post_json(
        "/posts/10/comments",
        Some(&token),
        &serde_json::json!({
            "body": "댓글입니다",
        }),
    )
    .await;

    // 알림 없음 확인
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(200)) => (),
        result = read.next() => {
            panic!("알림이 오면 안됨: {:?}", result);
        }
    }
}

/// 유효하지 않은 토큰으로 연결 시도(패닉)
#[tokio::test]
#[should_panic(expected = "WS 연결 실패")]
async fn test_invalid_token() {
    let ctx = common::TestContext::new().await;
    let _ = ctx.connect_ws("invalid", "/ws/notifications").await;
}
