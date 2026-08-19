use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use utoipa::OpenApi;

use crate::{
    auth::extractor::AuthUser,
    domain::comment::{CommentResponse, CreateCommentInput},
    error::{AppError, ErrorBody},
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
            description = "부모 게시글을 찾을 수 없음",
            body = ErrorBody,
            example = json!({ "error": "not_found", "message": "comment(id=1)를 찾을 수 없습니다" })
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY,
            description = "댓글이 비어 있거나 너무 긴 경우",
            body = ErrorBody,
            example = json!({ "error": "validation_error", "message": "댓글이 비어 있습니다" })
        )
    ),
    tags = ["comments"]
)]
pub async fn create_comment(
    auth_user: AuthUser,
    Path(post_id): Path<i64>,
    State(state): State<AppState>,
    Json(input): Json<CreateCommentInput>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    let comment = state
        .comment_service
        .create(post_id, input, auth_user.user_id)
        .await?;

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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            description = "내부 서버 오류",
            body = ErrorBody,
            example = json!({ "error": "internal_error", "message":  "서버 내부 오류가 발생했습니다" })
        )
    ),
    tags = ["comments"]
)]
pub async fn list_comments(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<CommentResponse>>, AppError> {
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
    components(schemas(CommentResponse, CreateCommentInput, ErrorBody, AppError))
)]
pub struct CommentOpenApiDoc;
