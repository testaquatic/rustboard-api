#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Notification {
    /// 이벤트의 종류: "comment_added", "comment_deleted" 등
    #[serde(rename = "type")]
    pub event_type: String,
    /// 대상 게시글 id
    pub post_id: i64,
    /// 관련 댓글 id (없을 수 있음)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<i64>,
    /// 행위자 이름
    pub actor: String,
    /// 사람이 읽을 수 있는 메시지
    pub message: String,
}

/// 클라이언트에서 보내는 메시지
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { post_id: i64 },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { post_id: i64 },
}

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast;

    use crate::notification::{ClientMessage, Notification};

    fn sample_notification(post_id: i64) -> Notification {
        Notification {
            event_type: "comment_added".to_string(),
            post_id,
            comment_id: Some(1),
            actor: "tester".to_string(),
            message: "테스트 알림".to_string(),
        }
    }

    #[tokio::test]
    async fn test_fanout_all_receivers_get_message() -> Result<(), anyhow::Error> {
        let (tx, _) = broadcast::channel(16);

        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();
        let mut rx3 = tx.subscribe();

        let notification = sample_notification(42);
        tx.send(notification.clone())?;

        let n1 = rx1.recv().await?;
        let n2 = rx2.recv().await?;
        let n3 = rx3.recv().await?;

        assert_eq!(n1.post_id, 42);
        assert_eq!(n2.post_id, 42);
        assert_eq!(n3.post_id, 42);

        Ok(())
    }

    #[tokio::test]
    async fn test_lagged_when_capacity_exceeded() -> Result<(), anyhow::Error> {
        // 용량 2인 채널
        let (tx, _) = broadcast::channel(2);
        let mut rx = tx.subscribe();

        // 3개 메시지 발행
        tx.send(sample_notification(1))?;
        tx.send(sample_notification(2))?;
        tx.send(sample_notification(3))?;

        // 첫 수신에서 Lagged 발생
        let result = rx.recv().await;
        match result {
            Err(broadcast::error::RecvError::Lagged(n)) => {
                assert_eq!(n, 1, "1개 메시지가 밀려남");
            }
            _ => panic!("Lagged가 발생해야 함"),
        }

        // 이후 메시지는 정상 수신
        let n = rx.recv().await?;
        assert_eq!(n.post_id, 2);

        let n = rx.recv().await?;
        assert_eq!(n.post_id, 3);

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use std::collections::HashSet;

        use crate::notification::tests::sample_notification;

        #[test]
        fn test_subscription_filter() {
            let mut subscribed = HashSet::<i64>::new();
            subscribed.insert(42);
            subscribed.insert(100);

            let notification42 = sample_notification(42);
            let notification99 = sample_notification(99);

            assert!(subscribed.contains(&notification42.post_id));
            assert!(!subscribed.contains(&notification99.post_id));
        }
    }

    #[tokio::test]
    async fn test_send_with_no_receivers() {
        let (tx, _) = broadcast::channel(16);

        let result = tx.send(sample_notification(42));
        assert!(result.is_err(), "수신자 없으면 SendErrr");
    }

    #[test]
    fn test_notification_serialization() -> Result<(), anyhow::Error> {
        let n = Notification {
            event_type: "comment_added".to_string(),
            post_id: 42,
            comment_id: Some(7),
            actor: "철수".to_string(),
            message: "철수님이 댓글을 달았습니다.".to_string(),
        };

        let json = serde_json::to_string(&n)?;
        let parsed = serde_json::from_str::<serde_json::Value>(&json)?;

        assert_eq!(parsed["post_id"], 42);
        assert_eq!(parsed["comment_id"], 7);
        assert_eq!(parsed["type"], "comment_added");
        assert_eq!(parsed["actor"], "철수");
        assert_eq!(parsed["message"], "철수님이 댓글을 달았습니다.");

        Ok(())
    }

    #[test]
    fn test_notification_without_comment_id() -> Result<(), anyhow::Error> {
        let n = Notification {
            event_type: "post_created".to_string(),
            post_id: 42,
            comment_id: None,
            actor: "영희".to_string(),
            message: "새 게시글이 등록되었습니다.".to_string(),
        };

        let json = serde_json::to_string(&n)?;
        let parsed = serde_json::from_str::<serde_json::Value>(&json)?;

        assert!(
            parsed.get("comment_id").is_none(),
            "comment_id가 None이면 JSON에서 빠져야 함"
        );

        Ok(())
    }

    #[test]
    fn test_client_message_subscribe() -> Result<(), anyhow::Error> {
        let json = serde_json::json!({"action":"subscribe","post_id": 42}).to_string();
        let msg = serde_json::from_str(&json)?;
        match msg {
            ClientMessage::Subscribe { post_id } => {
                assert_eq!(post_id, 42, "post_id 불일치");
            }
            _ => panic!("ClientMessage::Subscribe여야 함"),
        }

        Ok(())
    }

    #[test]
    fn test_client_message_unsubscribe() -> Result<(), anyhow::Error> {
        let json = serde_json::json!({"action":"unsubscribe","post_id": 42}).to_string();
        let msg = serde_json::from_str(&json)?;
        match msg {
            ClientMessage::Unsubscribe { post_id } => {
                assert_eq!(post_id, 42, "post_id 불일치");
            }
            _ => panic!("ClientMessage::Unsubscribe여야 함"),
        }

        Ok(())
    }

    #[test]
    fn test_client_message_invalid_action() {
        let json = serde_json::json!({"action":"invalid","post_id": 42}).to_string();
        let msg = serde_json::from_str::<ClientMessage>(&json);

        assert!(msg.is_err(), "잘못된 action은 에러를 반환해야 함");
    }
}
