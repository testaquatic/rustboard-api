use std::sync::Arc;

use crate::{
    domain::comment::{Comment, CreateCommentInput},
    repository::{comment::CommentRepository, posts::PostRepository},
    service::posts::ServiceError,
};

pub struct CommentService {
    posts_repo: Arc<dyn PostRepository + Send + Sync>,
    comments_repo: Arc<dyn CommentRepository + Send + Sync>,
}

impl CommentService {
    pub fn new(
        posts_repo: Arc<dyn PostRepository + Send + Sync>,
        comments_repo: Arc<dyn CommentRepository + Send + Sync>,
    ) -> Self {
        Self {
            posts_repo,
            comments_repo,
        }
    }

    pub async fn create(
        &self,
        post_id: i64,
        input: CreateCommentInput,
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

        self.comments_repo
            .insert(post_id, input)
            .await
            .map_err(From::from)
    }

    pub async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, ServiceError> {
        self.comments_repo
            .list_by_post(post_id)
            .await
            .map_err(From::from)
    }
}
