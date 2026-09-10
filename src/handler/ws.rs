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
    sync::{RwLock, broadcast},
    time::interval,
};
use utoipa::OpenApi;

use crate::{auth::extractor::AuthUser, domain::notification::ClientMessage, state::AppState};

#[utoipa::path(
    description = "웹소켓을 통해서 실시간으로 알림을 받는다",
    get,
    path = "/ws/notifications",
    security(("AuthUser" = ["read:notifications"])),
    tags = ["notifications"]
)]
pub async fn ws_notifications(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    auth_user: AuthUser,
) -> Response {
    ws.on_upgrade(move |socket| handle_notifications(socket, app_state, auth_user))
}

#[derive(OpenApi)]
#[openapi(
    paths(ws_notifications),
    tags((name = "notifications", description = "알림 API"))
)]
pub struct WsOpenApiDoc;

async fn handle_notifications(socket: WebSocket, state: AppState, auth_user: AuthUser) {
    let (mut sender, mut receiver) = socket.split();
    let mut notify_rx = state.notify_tx.subscribe();

    // 연결된 사용자가 구독 중인 게시글 ID 목록
    let subscribed_posts = Arc::new(RwLock::new(HashSet::<i64>::new()));

    let pong_received = Arc::new(AtomicBool::new(true));
    let pong_for_recv = pong_received.clone();
    let pong_for_send = pong_received.clone();

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
                Message::Pong(_) => {
                    pong_for_recv.store(true, Ordering::Relaxed);
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
        let mut heartbeat = interval(Duration::from_secs(10));
        // 첫 tick은 즉시 발생하므로 건너 뜀
        heartbeat.tick().await;

        loop {
            tokio::select! {
                // broadcast에서 알림 수신
                result = notify_rx.recv() => {
                    match result {
                        Ok(notification) => {
                            // 정상 전달
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
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(missing = n, "느린 소비자: {n}개 알림 누락");
                            // 계속 진행 - 이후 메시지부터 받음
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            // 채널 닫힘
                            break;
                        }
                }
            }
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
    };

    tracing::info!(user = %auth_user.name, "WebSocket 연결 종료");
}
