use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};

use crate::{
    domain::post::{CreatePostInput, PostResponse, ServiceError},
    state::AppState,
};

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {}

#[utoipa::path(
    description = "게시글 목록을 가져온다.",
    get,
    path = "/posts",
    params(
        ListQuery
    ),
    responses(
        (status = StatusCode::OK, description = "성공적으로 게시글의 목록을 반환", body = [PostResponse], example = json!([
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
        ]))
    )
)]
pub async fn list_posts(
    State(state): State<AppState>,
    _query: Query<ListQuery>,
) -> Result<Json<Vec<PostResponse>>, ServiceError> {
    let body = state
        .post_service
        .list_recent()
        .await?
        .into_iter()
        .map(PostResponse::from)
        .collect::<Vec<_>>();

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
        (status = StatusCode::NOT_FOUND, description = "게시글을 찾을 수 없음", body = ServiceError, example = json!({"message": ServiceError::NotFound(1).to_string()}))
    )
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PostResponse>, ServiceError> {
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
        (status = StatusCode::BAD_REQUEST, description = "잘못된 요청", example = json!({"message": ServiceError::EmptyTitle.to_string()}))
    )
)]
pub async fn create_post(
    State(state): State<AppState>,
    Json(input): Json<CreatePostInput>,
) -> Result<(StatusCode, Json<PostResponse>), ServiceError> {
    let post = state.post_service.create(input).await?;

    Ok((StatusCode::CREATED, Json(PostResponse::from(post))))
}

#[derive(OpenApi)]
#[openapi(
    paths(list_posts, get_post, create_post),
    components(schemas(CreatePostInput, ServiceError, PostResponse))
)]
pub struct PostOpenApiDoc;
