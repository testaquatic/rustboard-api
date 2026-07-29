use std::sync::Arc;

use crate::{
    domain::post::{CreatePostInput, Post, ServiceError},
    repository::post::PostRepository,
};

const TITLE_MAX: usize = 200;
const BODY_MAX: usize = 10_000;

pub struct PostService {
    repo: Arc<dyn PostRepository + Send + Sync>,
}

impl PostService {
    pub fn new(repo: Arc<dyn PostRepository + Send + Sync>) -> Self {
        PostService { repo }
    }

    pub async fn create(&self, input: CreatePostInput) -> Result<Post, ServiceError> {
        let title = input.title.trim();
        if title.is_empty() {
            return Err(ServiceError::EmptyTitle);
        }
        if title.chars().count() > TITLE_MAX {
            return Err(ServiceError::TitleTooLong(TITLE_MAX));
        }
        if input.body.chars().count() > BODY_MAX {
            return Err(ServiceError::BodyTooLong(BODY_MAX));
        }

        let clean = CreatePostInput {
            title: title.to_string(),
            body: input.body,
        };

        self.repo
            .insert(clean)
            .await
            .map_err(|_| ServiceError::Internal)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Post, ServiceError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|_| ServiceError::Internal)?
            .ok_or(ServiceError::NotFound(id))
    }
}
