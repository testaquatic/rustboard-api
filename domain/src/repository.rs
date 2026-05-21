use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    comment::{Comment, CreateCommentInput},
    error::RepositoryError,
    posts::{CreatePostInput, Post, UpdatePostInput},
    user::User,
};

#[async_trait]
pub trait CommentRepository {
    async fn insert(
        &self,
        post_id: i64,
        input: CreateCommentInput,
    ) -> Result<Comment, RepositoryError>;

    async fn list_by_post(&self, post_id: i64) -> Result<Vec<Comment>, RepositoryError>;

    async fn find_by_id(&self, id: i64) -> Result<Option<Comment>, RepositoryError>;

    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub type DynCommentRepository = Arc<dyn CommentRepository + Send + Sync>;

/// 저장소 트레이트
#[async_trait]
pub trait PostRepository {
    /// 저장소에 글을 저장한다.
    async fn insert(&self, input: CreatePostInput, author_id: i64)
    -> Result<Post, RepositoryError>;
    /// id를 기준으로 글을 불러온다.
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, RepositoryError>;

    /// 모든 글을 불러온다.
    async fn list(
        &self,
        cursor: Option<(DateTime<Utc>, i64)>,
        limit: i32,
    ) -> Result<Vec<Post>, RepositoryError>;
    /// 게시글을 수정한다.
    /// 해당 글이 없으면 Result<None>을 반환한다.
    async fn update(
        &self,
        id: i64,
        input: UpdatePostInput,
    ) -> Result<Option<Post>, RepositoryError>;
    /// 게시글을 삭제한다.
    async fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
}

pub type DynPostRepository = Arc<dyn PostRepository + Send + Sync>;

#[async_trait]
pub trait UserRepository {
    /// 회원 가입
    async fn insert(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User, RepositoryError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepositoryError>;
}
