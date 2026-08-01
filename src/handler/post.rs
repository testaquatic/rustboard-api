use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa::OpenApi;

use crate::{
    domain::post::{CreatePostInput, Post, ServiceError},
    state::AppState,
};

#[utoipa::path(
    description = "게시글 목록을 가져온다.",
    get,
    path = "/posts",
    responses(
        (status = 200, description = "ok", body = [Post], example = json!([
            Post{
                id: 1,
                title: "Post 1".to_string(),
                body: "Body 1".to_string(),
            },
            Post{
                id: 2,
                title: "Post 2".to_string(),
                body: "Body 2".to_string(),
            },
        ]))
    )
)]
pub async fn list_posts(State(state): State<AppState>) -> Result<Json<Vec<Post>>, ServiceError> {
    let posts = state.post_service.list().await?;

    Ok(Json(posts))
}

#[utoipa::path(
    description = "특정 id를 가진 게시글을 가져온다.",
    get,
    path = "/posts/{id}",
    params(
        ("id", description = "게시글 id")
    ),
    responses(
        (status = 200, description = "ok", body = Post),
        (status = 404, description = "게시글을 찾을 수 없습니다", body = ServiceError, example = json!({"message": ServiceError::NotFound(1).to_string()}))
    )
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Post>, ServiceError> {
    let post = state.post_service.get_by_id(id).await?;

    Ok(Json(post))
}

#[utoipa::path(
    description = "새로운 게시글을 작성한다.",
    post,
    path = "/posts",
    responses(
        (status = 201, description = "ok", body = Post),
        (status = 400, description = "잘못된 요청", body = ServiceError, example = json!({"message": ServiceError::EmptyTitle.to_string()}))
    )
)]
pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<Post>), ServiceError> {
    let post = state.post_service.create(payload).await?;

    Ok((StatusCode::CREATED, Json(post)))
}

#[derive(OpenApi)]
#[openapi(
    paths(list_posts, get_post, create_post),
    components(schemas(Post, CreatePostInput, ServiceError))
)]
pub struct PostOpenApiDoc;
