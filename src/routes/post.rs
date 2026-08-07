use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    domain::post::{CreatePostInput, PostResponse, UpdatePostInput},
    error::{AppError, ErrorBody},
    state::AppState,
};

const DEFAULT_LIMIT: i32 = 20;
const MAX_LIMIT: i32 = 100;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    /// 커서 문자열
    /// "{created_at}_{id}" 형태여야 한다.
    /// created_at는 유닉스 타임스탬프이다.
    #[into_params(required = false, example = Utc::now().timestamp())  ]
    pub cursor: Option<String>,
    #[into_params(required = false, default = DEFAULT_LIMIT, maximum = MAX_LIMIT, minimum = 1)]
    pub limit: Option<i32>,
}

/// 커서 문자열을 (created_at, id) 튜플로 변환한다.
fn parse_cursor(s: &str) -> Option<(DateTime<Utc>, i64)> {
    let (left, right) = s.split_once('_')?;
    let secs = left.parse::<i64>().ok()?;
    let id = right.parse::<i64>().ok()?;
    let ts = Utc.timestamp_opt(secs, 0).single()?;

    Some((ts, id))
}

/// cursor 문자열을 생성한다.
fn format_cursor(ts: DateTime<Utc>, id: i64) -> String {
    format!("{}_{}", ts.timestamp(), id)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PostListResponse {
    pub items: Vec<PostResponse>,
    pub next_cursor: Option<String>,
}

#[utoipa::path(
    description = "게시글 목록을 가져온다.",
    get,
    path = "/posts",
    params(ListQuery),
    responses(
        (
            status = StatusCode::OK,
            description = "성공적으로 게시글의 목록을 반환. 마지막 페이지는 next_cursor가 null",
            body = [PostListResponse],
            example = json!(
                PostListResponse{
                    items: vec![
                        PostResponse{
                            id: 1,
                            title: "Post 1".to_string(),
                            body: "Body 1".to_string(),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        },
                        PostResponse{
                            id: 2,
                            title: "Post 2".to_string(),
                            body: "Body 2".to_string(),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        },
                    ],
                    next_cursor: Some(format_cursor(Utc::now(), 2)),
                }
            )
        ),
    ),
    tags = ["posts"]
)]
pub async fn list_posts(
    State(state): State<AppState>,
    query: Query<ListQuery>,
) -> Result<Json<PostListResponse>, AppError> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let cursor = query.cursor.as_ref().and_then(|s| parse_cursor(s));
    let posts = state.post_service.list_recent(cursor, limit).await?;

    let next_cursor = posts.last().map(|p| format_cursor(p.created_at, p.id));
    let items = posts
        .into_iter()
        .map(PostResponse::from)
        .collect::<Vec<_>>();
    let body = PostListResponse { items, next_cursor };

    Ok(Json(body))
}

#[utoipa::path(
    description = "특정 id를 가진 게시글을 가져온다.",
    get,
    path = "/posts/{id}",
    params(
        ("id", description = "게시글 id")
    ),
    responses(
        (status = StatusCode::OK, description = "성공적으로 게시글을 반환", body = PostResponse),
        (
            status = StatusCode::NOT_FOUND,
            description = "게시글을 찾을 수 없음",
            body = ErrorBody,
            example = json!({"error": "not_found", "message": "post(id=1)를 찾을 수 없습니다"})
        )
    ),
    tags = ["posts"]
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PostResponse>, AppError> {
    let post = state.post_service.get_by_id(id).await?;

    Ok(Json(PostResponse::from(post)))
}

#[utoipa::path(
    description = "새로운 게시글을 작성한다.",
    post,
    path = "/posts",
    request_body = CreatePostInput,
    responses(
        (status = StatusCode::CREATED, description = "글 생성 성공", body = PostResponse),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY, description = "제목이나 내용이 없거나 너무 긴 경우", 
            body = ErrorBody,
            example = json!({"error": "validation_error", "message": "제목이 비어 있습니다"})
        )
    ),
    tags = ["posts"]
)]
pub async fn create_post(
    State(state): State<AppState>,
    Json(input): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<PostResponse>), AppError> {
    let post = state.post_service.create(input).await?;

    Ok((StatusCode::CREATED, Json(PostResponse::from(post))))
}

#[utoipa::path(
    description = "특정 id를 가진 게시글을 수정한다.",
    patch,
    path = "/posts/{id}",
    params(
        ("id", description = "게시글 id")
    ),
    request_body = UpdatePostInput,
    responses(
        (status = StatusCode::OK, description = "성공적으로 게시글을 수정", body = PostResponse),
        (
            status = StatusCode::NOT_FOUND, description = "게시글을 찾을 수 없음", 
            body = ErrorBody, example = json!({"error": "not_found", "message": "post(id=1)를 찾을 수 없습니다"})
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY, description = "제목이나 내용이 없거나 너무 긴 경우",
            body = ErrorBody, example = json!({"error": "validation_error", "message": "제목이 비어 있습니다"})
        )
    ),
    tags = ["posts"]
)]
pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<UpdatePostInput>,
) -> Result<Json<PostResponse>, AppError> {
    let post = state.post_service.update(id, input).await?;

    Ok(Json(PostResponse::from(post)))
}

#[utoipa::path(
    description = "특정 id를 가진 게시글을 삭제한다.",
    delete,
    path = "/posts/{id}",
    params(
        ("id", description = "게시글 id")
    ),
    responses(
        (status = StatusCode::NO_CONTENT, description = "성공적으로 게시글을 삭제"),
        (
            status = StatusCode::NOT_FOUND, description = "게시글을 찾을 수 없음", body = ErrorBody, 
            example = json!({"error": "not_found", "message": "post(id=1)를 찾을 수 없습니다"})
        ),
    ),
    tags = ["posts"]
)]
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.post_service.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(OpenApi)]
#[openapi(
    paths(list_posts, get_post, create_post, update_post, delete_post),
    tags((name = "posts", description = "게시글 API")),
    components(schemas(CreatePostInput, UpdatePostInput, AppError, ErrorBody, PostResponse))
)]
pub struct PostOpenApiDoc;
