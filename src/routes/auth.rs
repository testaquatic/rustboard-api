use axum::{Json, extract::State, http::StatusCode};
use utoipa::OpenApi;

use crate::{
    domain::user::{SignupInput, UserResponse},
    error::{AppError, ErrorBody},
    state::AppState,
};

#[utoipa::path(
    description = "회원가입",
    post,
    path = "/signup",
    request_body = SignupInput,
    responses(
        (status = 201, description = "회원 가입 성공", body = UserResponse),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY, description = "이미 존재하는 이메일", body = ErrorBody, 
            example = json!({"error": "validation_error", "message": "이미 존재하는 이메일입니다".to_string()})
        ),
        (
            status = 500, description = "내부 서버 오류", body = ErrorBody,
            example = json!({ "error": "internal_error", "message":  "서버 내부 오류가 발생했습니다" })
        )
    ),
    tags = ["auth"]
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(input): Json<SignupInput>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let user = state.user_service.signup(input).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "auth", description = "인증 API")),
    paths(signup),
    components(schemas(UserResponse, SignupInput, ErrorBody, AppError))
)]
pub struct AuthOpenApiDoc;
