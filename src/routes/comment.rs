use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa::OpenApi;

use crate::{
    domain::{
        comment::{CommentResponse, CreateCommentInput},
        post::ServiceError,
    },
    state::AppState,
};

#[utoipa::path(
    description = "댓글 생성",
    post,
    path = "/posts/{post_id}/comments",
    params(
        ("post_id", description = "게시글 id")
    ),
    request_body = CreateCommentInput,
    responses(
        (status = StatusCode::CREATED, description = "댓글 생성 성공", body = CommentResponse),
        (
            status = StatusCode::NOT_FOUND,
            description = "게시글을 찾을 수 없음",
            body = ServiceError,
            example = json!({ "error": ServiceError::NotFound(1).to_string() })
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "잘못된 요청",
            body = ServiceError,
            example = json!({ "error": ServiceError::EmptyTitle.to_string() })
        )
    ),
    tags = ["comments"]
)]
pub async fn create_comment(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Json(input): Json<CreateCommentInput>,
) -> Result<(StatusCode, Json<CommentResponse>), ServiceError> {
    let comment = state.comment_service.create(post_id, input).await?;

    Ok((StatusCode::CREATED, Json(CommentResponse::from(comment))))
}

#[utoipa::path(
    description = "댓글 목록 조회",
    get,
    path = "/posts/{post_id}/comments",
    params(
        ("post_id", description = "게시글 id")
    ),
    responses(
        (status = StatusCode::OK, description = "댓글 목록 조회 성공", body = Vec<CommentResponse>),
        (
            status = StatusCode::NOT_FOUND,
            description = "게시글을 찾을 수 없음",
            body = ServiceError,
            example = json!({ "error": ServiceError::NotFound(1).to_string() })
        )
    ),
    tags = ["comments"]
)]
pub async fn list_comments(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<CommentResponse>>, ServiceError> {
    let comments = state.comment_service.list_by_post(post_id).await?;
    let body = comments
        .into_iter()
        .map(CommentResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(body))
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "comments", description = "댓글 API")),
    paths(create_comment, list_comments),
    components(schemas(CommentResponse, CreateCommentInput, ServiceError))
)]
pub struct CommentOpenApiDoc;
