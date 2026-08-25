use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    configuration::Settings,
    repository::{
        comment::{CommentRepository, PostgresCommentRepository},
        post::{PostRepository, PostgresPostRepository},
        user::{PostgresUserRepository, UserRepository},
    },
    service::{comment::CommentService, post::PostService, user::UserService},
};

pub type PostgresAppState =
    AppState<PostgresPostRepository, PostgresCommentRepository, PostgresUserRepository>;

/// AppState 정의
#[derive(Clone)]
pub struct AppState<
    PostRepo: PostRepository,
    CommentRepo: CommentRepository,
    UserRepo: UserRepository,
> {
    pub configuration: Arc<Settings>,
    pub pool: PgPool,
    pub post_service: Arc<PostService<PostRepo>>,
    pub comment_service: Arc<CommentService<PostRepo, CommentRepo>>,
    pub user_service: Arc<UserService<UserRepo>>,
}
