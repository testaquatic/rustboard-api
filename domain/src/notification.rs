use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// 이벤트 종류
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// 대상 게시글 ID
    pub post_id: i64,
    /// 관련 댓글 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<i64>,
    /// 행위자 이름
    pub actor: String,
    /// 사람이 읽을 수 있는 메시지
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    #[serde(rename = "comment_added")]
    CommentAdded,
    #[serde(rename = "comment_deleted")]
    CommentDeleted,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { post_id: i64 },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { post_id: i64 },
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use tokio::sync::broadcast;

    use crate::notification::{ClientMessage, EventType, Notification};

    fn sample_notification(post_id: i64) -> Notification {
        Notification {
            event_type: super::EventType::CommentAdded,
            post_id,
            comment_id: Some(1),
            actor: "tester".into(),
            message: "테스트 알림".into(),
        }
    }

    #[tokio::test]
    async fn test_fanout_all_receivers_get_message() {
        let (tx, _) = broadcast::channel::<Notification>(16);

        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();
        let mut rx3 = tx.subscribe();

        tx.send(sample_notification(42)).unwrap();

        let n1 = rx1.recv().await.unwrap();
        let n2 = rx2.recv().await.unwrap();
        let n3 = rx3.recv().await.unwrap();

        assert_eq!(n1.post_id, 42);
        assert_eq!(n2.post_id, 42);
        assert_eq!(n3.post_id, 42);
    }

    #[tokio::test]
    async fn test_lagged_when_capacity_exceeded() {
        let (tx, _) = broadcast::channel::<Notification>(2);

        let mut rx = tx.subscribe();

        tx.send(sample_notification(1)).unwrap();
        tx.send(sample_notification(2)).unwrap();
        tx.send(sample_notification(3)).unwrap();

        let result = rx.recv().await;
        match result {
            Err(broadcast::error::RecvError::Lagged(n)) => assert_eq!(n, 1, "1개 메시지가 밀려남"),
            _ => panic!("Lagged가 발생해야 함"),
        }

        let n = rx.recv().await.unwrap();
        assert_eq!(n.post_id, 2);

        let n = rx.recv().await.unwrap();
        assert_eq!(n.post_id, 3);
    }

    #[test]
    fn test_subscription_filter() {
        let mut subscribed = HashSet::new();
        subscribed.insert(42);
        subscribed.insert(100);

        let notification_42 = sample_notification(42);
        let notification_99 = sample_notification(99);

        assert!(subscribed.contains(&notification_42.post_id));
        assert!(!subscribed.contains(&notification_99.post_id));
    }

    /// 수신자가 0명일 때 send 동작 확인
    #[tokio::test]
    async fn test_send_with_no_receiver() {
        let (tx, _) = broadcast::channel::<Notification>(16);

        let result = tx.send(sample_notification(42));

        assert!(result.is_err(), "수신자 없으면 SendError");
    }

    #[test]
    fn test_notification_serialization() {
        let n = Notification {
            event_type: EventType::CommentAdded,
            post_id: 42,
            comment_id: Some(7),
            actor: "철수".into(),
            message: "철수님이 댓글을 달았습니다".to_string(),
        };

        let json = serde_json::to_string(&n).unwrap();
        let parsed = serde_json::from_str::<serde_json::Value>(&json).unwrap();
        assert_eq!(parsed["type"], "comment_added");
        assert_eq!(parsed["post_id"], 42);
        assert_eq!(parsed["comment_id"], 7);
        assert_eq!(parsed["actor"], "철수");
        assert_eq!(parsed["message"], "철수님이 댓글을 달았습니다");
    }

    #[test]
    fn test_notification_without_comment_id() {
        let n = Notification {
            event_type: EventType::CommentAdded,
            post_id: 42,
            comment_id: None,
            actor: "영희".into(),
            message: "영희님이 댓글을 달았습니다".to_string(),
        };

        let json = serde_json::to_string(&n).unwrap();
        let parsed = serde_json::from_str::<serde_json::Value>(&json).unwrap();

        assert!(
            parsed.get("comment_id").is_none(),
            "comment_id가 None이면 JSON에서 빠져야 함"
        );
    }

    #[test]
    fn test_client_message_subscribed() {
        let json = r#"{"action": "subscribe", "post_id": 42}"#;
        let msg = serde_json::from_str::<ClientMessage>(json).unwrap();
        match msg {
            ClientMessage::Subscribe { post_id } => assert_eq!(post_id, 42),
            _ => panic!("`ClientMessage::Subscribe`가 아님"),
        }
    }

    #[test]
    fn test_client_message_unsubscribed() {
        let json = r#"{"action": "unsubscribe", "post_id": 42}"#;
        let msg = serde_json::from_str::<ClientMessage>(json).unwrap();
        match msg {
            ClientMessage::Unsubscribe { post_id } => assert_eq!(post_id, 42),
            _ => panic!("`ClientMessage::Unsubscribe`가 아님"),
        }
    }

    #[test]
    fn test_client_message_invalid_action() {
        let json = r#"{"action": "invalid", "post_id": 42}"#;
        let result = serde_json::from_str::<ClientMessage>(json);
        assert!(result.is_err(), "알 수 없는 action은 실패해야 함");
    }
}
