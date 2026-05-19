use std::sync::Arc;

use rustboard_domain::{
    comment::{Comment, CreateCommentInput},
    notification::{EventType, Notification},
    repository::{CommentRepository, PostRepository},
};
use tokio::sync::broadcast;

use crate::service::error::ServiceError;

pub struct CommentService {
    posts_repo: Arc<dyn PostRepository + Send + Sync>,
    comments_repo: Arc<dyn CommentRepository + Send + Sync>,
    notify_tx: broadcast::Sender<Notification>,
}

impl CommentService {
    pub fn new(
        posts_repo: Arc<dyn PostRepository + Send + Sync>,
        comments_repo: Arc<dyn CommentRepository + Send + Sync>,
        notify_tx: broadcast::Sender<Notification>,
    ) -> Self {
        Self {
            posts_repo,
            comments_repo,
            notify_tx,
        }
    }

    pub async fn create(
        &self,
        post_id: i64,
        input: CreateCommentInput,
        requester_id: i64,
        actor_name: &str,
    ) -> Result<Comment, ServiceError> {
        let _ = requester_id;
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

        // 댓글 생성 알림 발행
        let _ = self.notify_tx.send(Notification {
            event_type: EventType::CommentAdded,
            post_id,
            comment_id: Some(comment.id),
            actor: actor_name.to_string(),
            message: format!(
                "{}님이 {}번 게시글에 댓글을 달았습니다",
                actor_name, post_id
            ),
        });

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
