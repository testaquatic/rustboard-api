use std::sync::Arc;

use axum::Router;
use rustboard_api::{
    configuration::get_configuration,
    repository::{
        comment::PostgresCommentRepository, post::PostgresPostRepository,
        user::PostgresUserRepository,
    },
    service::{comment::CommentService, post::PostService, user::UserService},
    state::AppState,
};
use sqlx::{QueryBuilder, postgres::PgPoolOptions};
use uuid::Uuid;

pub async fn test_app() -> Router {
    let mut configuration = get_configuration().unwrap();
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "postgres://{}:{}@{}:{}",
            configuration.database.username,
            configuration.database.password,
            configuration.database.host,
            configuration.database.port,
        ))
        .await
        .expect("Failed to connect database");

    let mut query = QueryBuilder::new(format!(
        r#"CREATE DATABASE "{}";"#,
        configuration.database.database_name
    ));
    query.build().execute(&pool).await.expect("DB 생성 실패");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.database_url())
        .await
        .expect("Failed to connect database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("DB 마이그레이션 실패");

    let posts_repo = Arc::new(PostgresPostRepository::new(db_pool.clone()));
    let comments_repo = Arc::new(PostgresCommentRepository::new(db_pool.clone()));
    let users_repo = Arc::new(PostgresUserRepository::new(db_pool.clone()));

    let post_service = Arc::new(PostService::new(posts_repo.clone()));
    let comment_service = Arc::new(CommentService::new(posts_repo, comments_repo));
    let user_service = Arc::new(UserService::new(users_repo));

    let state = AppState {
        post_service,
        comment_service,
        configuration: Arc::new(configuration),
        pool: db_pool,
        user_service,
    };

    rustboard_api::router::create_router(state)
}
