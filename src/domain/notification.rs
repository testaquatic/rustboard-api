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
