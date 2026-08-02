use crate::{
    domain::{
        comment::{Comment, CreateCommentInput},
        post::ServiceError,
    },
    repository::{comment::DynCommentRepository, post::DynPostRepository},
};

pub struct CommentService {
    posts_repo: DynPostRepository,
    comments_repo: DynCommentRepository,
}

impl CommentService {
    pub fn new(posts_repo: DynPostRepository, comments_repo: DynCommentRepository) -> Self {
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
            return Err(ServiceError::EmptyTitle);
        }

        self.comments_repo
            .insert(post_id, input)
            .await
            .map_err(|_| ServiceError::Internal)
    }

    pub async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, ServiceError> {
        self.comments_repo
            .list_by_post(post_id)
            .await
            .map_err(|_| ServiceError::Internal)
    }
}
