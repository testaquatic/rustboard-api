use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    sync::{OwnedSemaphorePermit, RwLock, broadcast},
    time::interval,
};

use crate::{
    auth::extractor::AuthUser, domain::notification::ClientMessage, error::AppError,
    state::AppState,
};

pub async fn ws_notification(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    auth_user: AuthUser,
) -> Result<Response, AppError> {
    let permit = state
        .ws_semaphore
        .clone()
        .try_acquire_owned()
        .map_err(|_| AppError::TooMayConnections)?;

    let response_body =
        ws.on_upgrade(move |socket| handle_notification(socket, state, auth_user, permit));

    Ok(response_body)
}

async fn handle_notification(
    socket: WebSocket,
    state: AppState,
    auth_user: AuthUser,
    _permit: OwnedSemaphorePermit,
) {
    // _permit는 이 함수가 끝나면 자동으로 드롭 => 다른 연결이 들어 올 수 있음
    let (mut sender, mut receiver) = socket.split();
    let mut notify_rx = state.notify_tx.subscribe();

    // 연결된 사용자가 구독중인 게시글 ID 목록
    let subscribed_posts = Arc::new(RwLock::new(HashSet::new()));

    // 수신 태스크용 클론
    let subs_for_recv = subscribed_posts.clone();
    let username = auth_user.name.clone();

    let pong_received = Arc::new(AtomicBool::new(true));
    let pong_for_recv = pong_received.clone();
    let pong_for_send = pong_received.clone();

    // 수신 태스크
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        let mut subs = subs_for_recv.write().await;
                        match client_msg {
                            ClientMessage::Subscribe { post_id } => {
                                subs.insert(post_id);
                                tracing::debug!(user = %username, post_id, "게시글 구독");
                            }
                            ClientMessage::Unsubscribe { post_id } => {
                                subs.remove(&post_id);
                                tracing::debug!(user = %username, post_id, "게시글 구독 취소");
                            }
                        }
                    }
                }
                Message::Pong(_) => pong_for_recv.store(true, Ordering::Relaxed),
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // 송신 태스크
    let mut send_task = tokio::spawn(async move {
        let mut heartbeat = interval(Duration::from_secs(30));
        heartbeat.tick().await;

        loop {
            tokio::select! {
                result = notify_rx.recv() => {
                    match result {
                        Ok(notification) => {
                            let is_subscribed = {
                                let subs = subscribed_posts.read().await;
                                subs.contains(&notification.post_id)
                            };

                            if is_subscribed {
                                let json = serde_json::to_string(&notification).unwrap_or_default();
                                if sender.send(json.into()).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(missed = n, "느린 소비자: {n}개 알림 누적");
                            // 계속 진행
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                // 30초마다 Ping
                _ = heartbeat.tick() => {
                    if !pong_for_send.load(Ordering::Relaxed) {
                        tracing::warn!("Pong 미응답, 연결 종료");
                        break;
                    }
                    pong_for_send.store(false, Ordering::Relaxed);

                    if sender.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // 어느 한 쪽이 끝나면 나머지도 종료
    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }

    tracing::info!(user = %auth_user.name, "WebSocket 연결 종료");
}
