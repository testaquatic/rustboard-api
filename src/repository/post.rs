use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::post::{CreatePostInput, Post};

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("저장소 오류")]
    Backend,
}

#[async_trait]
pub trait PostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    async fn list(&self) -> Result<Vec<Post>, RepositoryError>;
}

pub type DynPostRepository = Arc<dyn PostRepository + Send + Sync>;
