use crate::{
    domain::post::{CreatePostInput, PostListResponse, PostRow},
    repository::post::PostsRepository,
    service::error::ServiceError,
};

pub struct PostsService {
    post_rep: PostsRepository,
}

impl PostsService {
    pub fn new(post_rep: PostsRepository) -> Self {
        Self { post_rep }
    }

    pub async fn list(&self, page: i64, limit: i64) -> Result<PostListResponse, ServiceError> {
        let (posts, total) = self.post_rep.list(page, limit).await?;

        let post_list_response = PostListResponse {
            posts: posts.into_iter().map(|post| post.into()).collect(),
            total,
            page,
        };

        Ok(post_list_response)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<PostRow, ServiceError> {
        let Some(post_response) = self.post_rep.find_by_id(id).await? else {
            return Err(ServiceError::NotFound {
                entity: "posts".to_string(),
                id,
            });
        };

        Ok(post_response)
    }

    pub async fn create_post(
        &self,
        create_post_input: &CreatePostInput,
        author_id: i64,
    ) -> Result<PostRow, ServiceError> {
        let post_response = self
            .post_rep
            .create(
                &create_post_input.title,
                &create_post_input.content,
                author_id,
            )
            .await?;

        Ok(post_response)
    }
}
