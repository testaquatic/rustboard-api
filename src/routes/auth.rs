use axum::{Json, extract::State, http::StatusCode};

use crate::{
    domain::user::{SignupInput, UserResponse},
    error::{AppError, ErrorBody},
    state::AppState,
};

/// 회원가입 핸들러
#[utoipa::path(
    post,
    tag = "auth",
    path = "/signup",
    summary = "회원가입",
    description = "회원가입 엔드포인트",
    request_body = SignupInput,
    responses((
        status = StatusCode::CREATED,
        content_type = "application/json",
        body = UserResponse
    ),(
        status = StatusCode::BAD_REQUEST,
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(input): Json<SignupInput>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = state.users_service.signup(input).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}


#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
pub struct AuthOpenApi;
