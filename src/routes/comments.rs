use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    domain::comment::{CommentResponse, CreateCommentInput}, error::{AppError, ErrorBody}, state::AppState
};


/// 댓글을 생성하는 핸들러
#[utoipa::path(
    post,
    tag = "comments",
    path = "/posts/{post_id}/comments",
    summary = "댓글 생성",
    description = "댓글을 생성하는 엔드포인트",
    params((
        "post_id" = i64, Path, description = "게시글 id"
    )),
    request_body = CreateCommentInput,
    responses((
        status = StatusCode::CREATED,
        content_type = "application/json",
        body = CommentResponse
    ),(
        status = StatusCode::BAD_REQUEST,
        content_type = "application/json",
        body = ErrorBody,
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
pub async fn create_comment(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Json(input): Json<CreateCommentInput>,
) -> Result<(StatusCode, Json<CommentResponse>), AppError> {
    let comment = state.comments_service.create(post_id, input).await?;
    
    Ok((StatusCode::CREATED, Json(comment.into())))
}

/// 댓글 목록을 조회하는 핸들러
#[utoipa::path(
    get,
    tag = "comments",
    path = "/posts/{post_id}/comments",
    summary = "댓글 목록 조회",
    description = "댓글 목록을 조회하는 엔드포인트",
    params((
        "post_id" = i64, Path, description = "게시글 id"
    )),
    responses((
        status = StatusCode::OK,
        content_type = "application/json",
        body = Vec<CommentResponse>
    ),(
        status = StatusCode::INTERNAL_SERVER_ERROR,
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn list_comments(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<CommentResponse>>, AppError> {
  let comments = state.comments_service.list_by_post(post_id).await?;
  let body = comments.into_iter().map(CommentResponse::from).collect();

  Ok(Json(body))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(create_comment, list_comments))]
pub struct CommentsOpenApi;