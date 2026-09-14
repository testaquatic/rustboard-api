use rustboard_domain::{comment::Comment, error::service_error::ServiceError};

use crate::{
    client::notification::NotificationClient,
    handler::types::input::CreateCommentInput,
    repository::{comment::CommentRepository, post::PostRepository},
};

pub struct CommentService {
    posts_repo: PostRepository,
    comments_repo: CommentRepository,
    notification_client: NotificationClient,
}

impl CommentService {
    pub fn new(
        posts_repo: PostRepository,
        comments_repo: CommentRepository,
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

        // gRPC로 알림 전송 (실패해도 댓글 생성은 유지)
        if let Err(e) = self
            .notification_client
            .send_comment_notification(
                &comment.post_id.to_string(),
                actor_name,
                post_id,
                comment.id,
            )
            .await
        {
            tracing::warn!(error = %e, post_id, "알림 전송 실패 (댓글은 정상 생성됨)");
        }

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
