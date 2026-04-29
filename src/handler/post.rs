use axum::{
    Json, extract::{Path, State}, http::StatusCode
};

use crate::{
    domain::post::{CreatePostInput, ErrorBody, Post, ServiceError},
    state::AppState,
};

/// 게시글 목록 조회 엔드포인트
#[utoipa::path(
  get,
  path = "/posts",
  summary = "게시글 목록 조회",
  description = "게시글 목록을 조회하는 엔드포인트",
  responses(
    (
      status = 200,
      description = "정상",
      body = Vec<Post>
    )
  )
)]
pub async fn list_posts(State(state): State<AppState>) -> Result<Json<Vec<Post>>, ServiceError> {
    let posts = state.posts_service.list().await?;
    Ok(Json(posts))
}



/// 게시글을 id를 기준으로 조회하는 엔드포인트
#[utoipa::path(
  get,
  path = "/posts/{id}",
  summary = "게시글 조회",
  description = "id를 기준으로 게시글을 조회하는 엔드포인트",
  params(
    ("id"  = i64, Path, description = "게시글 id")
  ),
  responses(
    (
      status = 200,
      description = "정상",
      body = Post
    ),
    (
      status = StatusCode::NOT_FOUND,
      description = "게시글을 찾을 수 없습니다",
      body = ErrorBody,
    ) 
  )
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Post>, ServiceError> {
    let post = state
        .posts_service
        .get_by_id(id)
        .await?;
      
    Ok(Json(post))
}


/// 게시글 생성 엔드포인트
#[utoipa::path(
  post,
  path = "/posts",
  summary = "게시글 생성",
  description = "게시글을 생성하는 엔드포인트",
  request_body = CreatePostInput,
  responses(
    (
      status = StatusCode::CREATED,
      description = "정상",
      body = Post
    ),
    (
      status = StatusCode::BAD_REQUEST,
      description = "잘못된 요청",
      content_type = "application/json",
      body = ErrorBody,
    ),
  )
)]
pub async fn create_post(
    State(state): State<AppState>,
    Json(input): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<Post>), ServiceError> {
    let post = state.posts_service.create(input).await?;
    Ok((StatusCode::CREATED, Json(post)))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_posts, get_post, create_post), components(schemas(Post)))]
pub struct PostsOpenApi;