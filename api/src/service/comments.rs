use std::sync::Arc;

use rustboard_domain::comment::{Comment, CreateCommentInput};

use crate::{
    client::notification::NotificationClient,
    repository::types::{CommentRepository, PostRepository},
    service::error::ServiceError,
};

pub struct CommentService {
    posts_repo: Arc<dyn PostRepository + Send + Sync>,
    comments_repo: Arc<dyn CommentRepository + Send + Sync>,
    notification_client: NotificationClient,
}

impl CommentService {
    pub fn new(
        posts_repo: Arc<dyn PostRepository + Send + Sync>,
        comments_repo: Arc<dyn CommentRepository + Send + Sync>,
        notification_client: NotificationClient,
    ) -> Self {
        Self {
            posts_repo,
            comments_repo,
            notification_client,
        }
    }

    pub async fn create(
        &self,
        post_id: i64,
        input: CreateCommentInput,
        actor_name: &str,
    ) -> Result<Comment, ServiceError> {
        if input.body.trim().is_empty() {
            return Err(ServiceError::Validation("내용이 없습니다".into()));
        }

        // 부모 게시글이 있는지 먼저 확인
        let Some(_) = self.posts_repo.find_by_id(post_id).await? else {
            return Err(ServiceError::NotFound {
                entity: "post".into(),
                id: post_id,
            });
        };

        let comment = self.comments_repo.insert(post_id, input).await?;

        // gRPC로 알림 전송
        if let Err(e) = self
            .notification_client
            .send_comment_notificaiton(
                &comment.post_author_id.to_string(),
                actor_name,
                post_id,
                comment.id,
            )
            .await
        {
            tracing::warn!(error = %e, post_id, "알림 전송 실패 (댓글은 정상 생성됨)")
        }

        Ok(comment)
    }

    pub async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, ServiceError> {
        self.comments_repo
            .list_by_post(post_id)
            .await
            .map_err(From::from)
    }

    pub async fn delete_comment(&self, comment_id: i64) -> Result<(), ServiceError> {
        self.comments_repo.delete(comment_id).await?;

        Ok(())
    }
}
