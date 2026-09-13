use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rustboard_api::handler::types::input::CreatePostInput;
use rustboard_domain::notification::{ClientMessage, Notification};
use tokio_tungstenite::tungstenite::Message;

use crate::common::test_context::TestContext;

#[tokio::test]
async fn test_websocket_notifications() {
    let ctx = TestContext::new().await;
    // 토큰 발급
    let token = ctx.signup_and_login().await.unwrap();

    let (mut write, mut read) = ctx
        .connect_ws("/ws/notifications", Some(&token))
        .await
        .unwrap();

    // 게시글 구독
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
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
    let notification = tokio::time::timeout(Duration::from_secs(5), read.next())
        .await
        .expect("타임아웃")
        .unwrap()
        .unwrap();

    if let Message::Text(text) = notification {
        let notification =
            serde_json::from_str::<Notification>(text.as_str()).expect("JSON 변환 실패");

        assert_eq!(notification.post_id, 10);
        assert!(
            notification
                .message
                .contains("10번 게시글에 댓글을 달았습니다"),
            "알림 메시지가 다릅니다: {}",
            notification.message
        );
    } else {
        panic!("받은 메시지가 텍스트가 아님: {:?}", notification);
    }

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
        _ = tokio::time::sleep(Duration::from_millis(500)) => (),
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
        _ = tokio::time::sleep(Duration::from_millis(500)) => (),
        result = read.next() => {
            panic!("알림이 오면 안됨: {:?}", result);
        }
    }
}

/// 유효하지 않은 토큰으로 연결 시도(패닉)
#[tokio::test]
async fn test_ws_without_auth_rejected() {
    let ctx = TestContext::new().await;
    assert!(ctx.connect_ws("/ws/notifications", None).await.is_err());
    assert!(
        ctx.connect_ws("/ws/notifications", Some("test"))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_comment_notification() {
    let ctx = TestContext::new().await;
    let token = ctx.signup_and_login().await.expect("JWT 토큰 획득 실패");

    // WebSocket 연결
    let (mut writer, mut reader) = ctx
        .connect_ws("/ws/notifications", Some(&token))
        .await
        .unwrap();

    // 42번 게시글 구독
    writer
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&ClientMessage::Subscribe { post_id: 42 })
                .expect("JSON 변환 실패")
                .into(),
        ))
        .await
        .expect("구독 메시지 전송 실패");

    // 잠시 대기
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 글 생성
    ctx.seed_post(
        &(1..100)
            .map(|i| CreatePostInput {
                title: format!("{}번 글", i),
                content: "본문".to_string(),
            })
            .collect::<Vec<_>>(),
        &token,
    )
    .await;

    // 댓글 생성
    ctx.post_json(
        "/posts/42/comments",
        Some(&token),
        &serde_json::json!(
            {
                "body": "댓글입니다",
            }
        ),
    )
    .await;

    // 알림 수신
    let msg = tokio::time::timeout(Duration::from_secs(5), reader.next())
        .await
        .expect("타임아웃")
        .unwrap()
        .unwrap();

    if let Message::Text(text) = msg {
        let notification = serde_json::from_str::<Notification>(text.as_str()).unwrap();
        assert_eq!(notification.event_type, "comment_added");
        assert_eq!(notification.post_id, 42);
        assert!(notification.message.contains("댓글"));
    } else {
        panic!("받은 메시지가 텍스트가 아님: {:?}", msg);
    }

    writer.close().await.unwrap();
}
