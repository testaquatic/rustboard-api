use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;

use crate::{auth::extractor::AuthUser, domain::notification::ClientMessage, state::AppState};

pub async fn ws_notifications(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    auth_user: AuthUser,
) -> Response {
    ws.on_upgrade(move |socket| handle_notifications(socket, app_state, auth_user))
}

async fn handle_notifications(socket: WebSocket, state: AppState, auth_user: AuthUser) {
    let (mut sender, mut receiver) = socket.split();
    let mut notify_rx = state.notify_tx.subscribe();

    // 연결된 사용자가 구독 중인 게시글 ID 목록
    let subscribed_posts = Arc::new(RwLock::new(HashSet::<i64>::new()));

    // 수신 태스크용 클론
    let subs_for_recv = subscribed_posts.clone();
    let auth_user_for_recv = auth_user.clone();

    // 수신 태스크: 클라이언트의 구독/해제 요청 처리
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        let mut subs = subs_for_recv.write().await;
                        match client_msg {
                            ClientMessage::Subscribe { post_id } => {
                                subs.insert(post_id);
                                tracing::debug!(user = %auth_user_for_recv.name, post_id, "게시글 구독");
                            }
                            ClientMessage::Unsubscribe { post_id } => {
                                subs.remove(&post_id);
                                tracing::debug!(user = %auth_user_for_recv.name, post_id, "게시글 구독 해제");
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => (),
            }
        }
    });

    // 송신 태스크용 클론
    let subs_for_send = subscribed_posts.clone();

    // 송신 태스크: broadcast에서 알림을 받아 클라이언트에 전달
    let mut send_task = tokio::spawn(async move {
        while let Ok(notification) = notify_rx.recv().await {
            // 구독중인 게시글의 알림만 전달
            let is_subscribed = {
                let subs = subs_for_send.read().await;
                subs.contains(&notification.post_id)
            };

            if is_subscribed {
                let json = serde_json::to_string(&notification).unwrap_or_default();
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // 어느 한 쪽이 끝나면 나머지도 종료
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    };

    tracing::info!(user = %auth_user.name, "WebSocket 연결 종료");
}
