use rustboard_proto::notification::{
    BatchNotificationRequest, BatchNotificationResponse, NotificationRequest, NotificationResponse,
    NotificationType, notification_service_server::NotificationService,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct NotifierService;

impl NotifierService {
    pub fn new() -> Self {
        Self
    }
}

#[tonic::async_trait]
impl NotificationService for NotifierService {
    async fn send(
        &self,
        request: Request<NotificationRequest>,
    ) -> Result<Response<NotificationResponse>, Status> {
        let req = request.into_inner();

        if req.recipient_id.is_empty() {
            return Err(Status::invalid_argument("recipient_id is required"));
        }

        let notification_type = NotificationType::try_from(req.notification_type)
            .unwrap_or(NotificationType::Unspecified);

        tracing::info!(
          recipient = %req.recipient_id,
          sender = %req.sender_name,
          notification_type = ?notification_type,
          "알림 전송"
        );

        // 실제 알림 전송 로직
        let message_id = Uuid::new_v4().to_string();

        Ok(Response::new(NotificationResponse {
            success: true,
            message_id,
            error_message: String::new(),
        }))
    }

    async fn send_batch(
        &self,
        request: Request<BatchNotificationRequest>,
    ) -> Result<Response<BatchNotificationResponse>, Status> {
        let req = request.into_inner();
        let total = req.notifications.len() as i32;
        let mut succeeded = 0i32;
        let mut failed = 0i32;
        let mut failed_ids = Vec::new();

        for notification in req.notifications {
            if notification.recipient_id.is_empty() {
                failed = failed + 1;
                failed_ids.push(notification.recipient_id.clone());
                tracing::warn!(recipient = %notification.recipient_id, "빈 recipient_id, 건너뜀");
                continue;
            }

            tracing::info!(
              recipient = %notification.recipient_id,
              sender = %notification.sender_name,
              "배치 알림 전송"
            );
            succeeded = succeeded + 1;
        }

        Ok(Response::new(BatchNotificationResponse {
            total,
            succeeded,
            failed,
            failed_ids,
        }))
    }
}
