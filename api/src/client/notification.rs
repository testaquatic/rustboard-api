use rustboard_proto::notification::{
    NotificationRequest, NotificationResponse, NotificationType,
    notification_service_client::NotificationServiceClient,
};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct NotificationClient {
    inner: NotificationServiceClient<Channel>,
}

impl NotificationClient {
    pub async fn connect(addr: &str) -> Result<Self, tonic::transport::Error> {
        let inner = NotificationServiceClient::connect(addr.to_string()).await?;

        tracing::info!("gRPC 알림 클라이언트 연결");

        Ok(Self { inner })
    }

    pub async fn send_comment_notificaiton(
        &self,
        recipient_id: &str,
        sender_name: &str,
        post_id: i64,
        comment_id: i64,
    ) -> Result<NotificationResponse, tonic::Status> {
        let mut client = self.inner.clone();

        let request = tonic::Request::new(NotificationRequest {
            recipient_id: recipient_id.into(),
            sender_name: sender_name.into(),
            notification_type: NotificationType::CommentAdded.into(),
            title: "새 댓글".to_string(),
            body: format!("{}님이 댓글을 달았습니다.", sender_name),
            metadata: [
                ("post_id".into(), post_id.to_string()),
                ("comment_id".into(), comment_id.to_string()),
            ]
            .into(),
        });

        let response = client.send(request).await?;

        Ok(response.into_inner())
    }
}
