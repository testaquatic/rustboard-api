use crate::{
    domain::comment::{Comment, CreateCommentInput},
    repository::{comment::DynCommentRepository, post::DynPostRepository},
    service::error::ServiceError,
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

        Ok(comment)
    }

    pub async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, ServiceError> {
        let comments = self.comments_repo.list_by_post(post_id).await?;

        Ok(comments)
    }
}
