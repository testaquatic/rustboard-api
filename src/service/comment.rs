use tokio::sync::broadcast;

use crate::{
    domain::{
        comment::{Comment, CreateCommentInput},
        notification::Notification,
    },
    repository::{comment::CommentRepository, post::PostRepository},
    service::error::ServiceError,
};

pub struct CommentService {
    posts_repo: PostRepository,
    comments_repo: CommentRepository,
    notify_tx: broadcast::Sender<Notification>,
}

impl CommentService {
    pub fn new(
        posts_repo: PostRepository,
        comments_repo: CommentRepository,
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
        actor_name: &str,
    ) -> Result<Comment, ServiceError> {
        if input.body.trim().is_empty() {
            return Err(ServiceError::Validation("댓글이 비어 있습니다".to_string()));
        }

        // 부모 게시글이 존재하는지 확인한다.
        let parent = self.posts_repo.find_by_id(post_id).await?;
        if parent.is_none() {
            return Err(ServiceError::NotFound {
                entity: "comment",
                id: post_id,
            });
        }

        let comment = self.comments_repo.insert(post_id, input).await?;

        // 댓글 생성 알림 발생 (수신자가 없어도 무시)
        let _ = self.notify_tx.send(Notification {
            event_type: "comment_added".to_string(),
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
        let comments = self.comments_repo.list_by_post(post_id).await?;

        Ok(comments)
    }

    // pub async fn delete_comment(
    //     &self,
    //     comment_id: i64,
    //     requester_id: i64,
    //     requester_role: &Role,
    // ) -> Result<(), ServiceError> {
    //     let comment =
    //         self.comments_repo
    //             .find_by_id(comment_id)
    //             .await?
    //             .ok_or(ServiceError::NotFound {
    //                 entity: "comment",
    //                 id: comment_id,
    //             })?;

    //     // 본인 또는 어드민만 삭제 가능
    //     // check_ownership(comment.author_id, requester_id, requester_role)?;

    //     self.comments_repo.delete(comment_id).await?;

    //     Ok(())
    // }
}
