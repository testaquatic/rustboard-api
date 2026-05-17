use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use serde_json::json;
use tokio::time;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

use crate::common::server::TestServer;

#[tokio::test]
async fn ws_notification_test() {
    // 테스트용 DB를 가동한다.
    let test_server = TestServer::new().await;
    let reqwest_client = reqwest::Client::new();

    // 회원 가입
    let signup_body = json!( {
        "email": "test@example.com",
        "password": "password123",
        "display_name": "Tester",
    });

    // 회원 가입
    let response = reqwest_client
        .post(format!("http://{}/signup", test_server.addr))
        .header("Content-Type", "application/json")
        .json(&signup_body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let login_body = json!( {
        "email": "test@example.com",
        "password": "password123",
    });

    // 로그인
    let token = reqwest_client
        .post(format!("http://{}/login", test_server.addr))
        .header("Content-Type", "application/json")
        .json(&login_body)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();

    // 게시글 작성
    let post_body2 = json!({
      "title": "Test Post",
      "content": "This is a test post."
    });
    let response2 = reqwest_client
        .post(format!("http://{}/posts", test_server.addr))
        .header("Content-Type", "application/json")
        .bearer_auth(&token)
        .json(&post_body2)
        .send()
        .await
        .unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);
    let post_id2 = response2.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    let post_body = json!({
      "title": "Test Post",
      "content": "This is a test post."
    });
    let response = reqwest_client
        .post(format!("http://{}/posts", test_server.addr))
        .header("Content-Type", "application/json")
        .bearer_auth(&token)
        .json(&post_body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let post_id = response.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    // WebSocket 연결
    let mut ws_request = format!("ws://{}/ws/notifications", test_server.addr)
        .into_client_request()
        .unwrap();
    ws_request.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    let (ws_stream, _) = connect_async(ws_request).await.unwrap();
    let (mut tx, mut rx) = ws_stream.split();
    let subscription_body = json!({
        "action": "subscribe",
        "post_id": post_id,
    });

    // 구독 신청
    tx.send(Message::Text(subscription_body.to_string().into()))
        .await
        .unwrap();

    // 댓글 작성

    // 댓글 작성
    let comment_body2 = json!({
        "body": "This is a test comment."
    });
    reqwest_client
        .post(format!(
            "http://{}/posts/{}/comments",
            test_server.addr, post_id2
        ))
        .bearer_auth(&token)
        .json(&comment_body2)
        .send()
        .await
        .unwrap();

    let comment_body = json!({
        "body": "This is a test comment."
    });
    reqwest_client
        .post(format!(
            "http://{}/posts/{}/comments",
            test_server.addr, post_id
        ))
        .bearer_auth(&token)
        .json(&comment_body)
        .send()
        .await
        .unwrap();

    // 알림 수신 확인
    if let Message::Text(msg) = rx.next().await.unwrap().unwrap() {
        let notification = serde_json::from_str::<serde_json::Value>(&msg).unwrap();
        assert_eq!(notification["post_id"], post_id);
        assert_eq!(notification["type"], "comment_added");
    } else {
        panic!("알림 수신 실패");
    }

    tokio::select! {
      _ = rx.next() => panic!("알림 수신 실패"),
      _ = time::sleep(Duration::from_secs(1)) => {}
    }

    // 구독 취소
    let subscription_body = json!({
        "action": "unsubscribe",
        "post_id": post_id,
    });
    tx.send(Message::Text(subscription_body.to_string().into()))
        .await
        .unwrap();

    tokio::select! {
      _ = rx.next() => panic!("알림 수신 실패"),
      _ = time::sleep(Duration::from_secs(1)) => {}
    }

    // 토큰 없이 구독 시도
    // WebSocket 연결
    let ws_request = format!("ws://{}/ws/notifications", test_server.addr)
        .into_client_request()
        .unwrap();
    let Err(e) = connect_async(ws_request).await else {
        panic!("unexpected success!")
    };

    match e {
        tokio_tungstenite::tungstenite::Error::Http(e) => {
            assert_eq!(e.status(), StatusCode::UNAUTHORIZED)
        }
        _ => panic!("unexpected error"),
    }
}
