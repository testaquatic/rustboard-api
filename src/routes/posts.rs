use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    domain::posts::{CreatePostInput, Post, PostResponse, UpdatePostInput},
    service::posts::{ErrorBody, ServiceError},
    state::AppState,
};

/// GET /posts의 쿼리 파라미터
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    /// unix타임스탬프_id의 형식이다.
    pub cursor: Option<String>,
    /// 글의 수
    pub limit: Option<i32>,
}

const DEFAULT_LIMIT: i32 = 20;
const MAX_LIMIT: i32 = 100;

fn parse_cursor(s: &str) -> Option<(DateTime<Utc>, i64)> {
    let (left, right) = s.split_once('_')?;
    let secs = left.parse::<i64>().ok()?;
    let id = right.parse::<i64>().ok()?;
    let ts = Utc.timestamp_opt(secs, 0).single()?;
    Some((ts, id))
}

fn format_cursor(ts: DateTime<Utc>, id: i64) -> String {
    format!("{}_{}", ts.timestamp(), id)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostListResponse {
    pub posts: Vec<PostResponse>,
    pub next_cursor: Option<String>,
}

/// 게시글 목록 조회 엔드포인트
#[utoipa::path(
    get,
    tag = "posts",
    path = "/posts",
    summary = "게시글 목록 조회",
    description = "게시글 목록을 조회하는 엔드포인트",
    params(
        ListQuery
    ),
    responses((
        status = StatusCode::OK,
        content_type = "application/json",
        body = PostListResponse,
    ),(
        status = StatusCode::INTERNAL_SERVER_ERROR,
        content_type = "application/json",
        body = ErrorBody
    ))
)]
pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<PostListResponse>, ServiceError> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let cursor = query.cursor.as_deref().and_then(parse_cursor);

    let posts = state.posts_service.list_recent(cursor, limit).await?;

    let next_cursor = posts.last().map(|p| format_cursor(p.created_at, p.id));
    let posts = posts.into_iter().map(PostResponse::from).collect();
    Ok(Json(PostListResponse { posts, next_cursor }))
}

/// 게시글을 id를 기준으로 조회하는 엔드포인트
#[utoipa::path(
    get,
    tag = "posts",
    path = "/posts/{id}",
    summary = "게시글 조회",
    description = "id를 기준으로 게시글을 조회하는 엔드포인트",
    params((
        "id"  = i64, Path, description = "게시글 id")
    ),
    responses((
        status = StatusCode::OK,
        content_type = "application/json",
        body = Post
    ),(
        status = StatusCode::NOT_FOUND,
        content_type = "application/json",
        body = ErrorBody,
    ),(
        status = StatusCode::INTERNAL_SERVER_ERROR,
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PostResponse>, ServiceError> {
    let post = state.posts_service.get_by_id(id).await?;

    Ok(Json(post.into()))
}

/// 게시글 생성 엔드포인트
#[utoipa::path(
    post,
    tag = "posts",
    path = "/posts",
    summary = "게시글 생성",
    description = "게시글을 생성하는 엔드포인트",
    request_body = CreatePostInput,
    responses((
        status = StatusCode::CREATED,
        content_type = "application/json",
        body = PostResponse
    ),(
        status = StatusCode::BAD_REQUEST,
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn create_post(
    State(state): State<AppState>,
    Json(input): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<PostResponse>), ServiceError> {
    let post = state.posts_service.create(input).await?;
    Ok((StatusCode::CREATED, Json(post.into())))
}

#[utoipa::path(
    patch,
    tag = "posts",
    path = "/posts/{id}",
    summary = "게시글 수정",
    description = "게시글을 수정하는 엔드포인트",
    params((
        "id" = i64, Path, description = "게시글 id"
    )),
    request_body = UpdatePostInput,
    responses((
        status = StatusCode::OK,
        content_type = "application/json",
        body = PostResponse,
    ),(
        status = StatusCode::NOT_FOUND,
        content_type = "application/json",
        body = ErrorBody,
    ),(
        status = StatusCode::INTERNAL_SERVER_ERROR,
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdatePostInput>,
) -> Result<Json<PostResponse>, ServiceError> {
    let post = state.posts_service.update(id, input).await?;
    Ok(Json(post.into()))
}

#[utoipa::path(
    delete,
    tag = "posts",
    path = "/posts/{id}",
    summary = "게시글 삭제",
    description = "게시글을 삭제하는 엔드포인트",
    params((
        "id" = i64, Path, description = "게시글 id")
    ),
    responses((
        status = StatusCode::NO_CONTENT,
        description = "삭제 완료",
    ),(
        status = StatusCode::NOT_FOUND,
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ServiceError> {
    state.posts_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_posts, get_post, create_post, update_post, delete_post))]
pub struct PostsOpenApi;
