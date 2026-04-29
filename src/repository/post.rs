use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::domain::post::{CreatePostInput, Post};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("저장소 오류")]
    Backend,
}

/// 저장소
#[async_trait]
pub trait PostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;
    async fn list(&self) -> Result<Vec<Post>, RepositoryError>;
}

#[derive(Default)]
struct InMemoryState {
    next_id: i64,
    items: HashMap<i64, Post>,
}

#[derive(Default)]
pub struct InMemoryPostRepository {
    inner: Mutex<InMemoryState>,
}

impl InMemoryPostRepository {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InMemoryState::default()),
        }
    }
}

#[async_trait]
impl PostRepository for InMemoryPostRepository {
    async fn insert(&self, input: CreatePostInput) -> Result<Post, RepositoryError> {
        let mut state = self.inner.lock().await;
        let id = state.next_id;
        state.next_id += 1;
        let post = Post {
            id,
            title: input.title,
            body: input.body,
        };
        state.items.insert(id, post.clone());

        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        Ok(state.items.get(&id).cloned())
    }

    async fn list(&self) -> Result<Vec<Post>, RepositoryError> {
        let state = self.inner.lock().await;
        let mut item = state.items.values().cloned().collect::<Vec<_>>();
        item.sort_by_key(|p| p.id);
        Ok(item)
    }
}
