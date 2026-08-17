use axum::{Json, extract::State, http::StatusCode};
use utoipa::OpenApi;

use crate::{
    auth::jwt,
    domain::user::{LoginInput, SignupInput, TokenResponse, UserResponse},
    error::{AppError, ErrorBody},
    state::AppState,
};

#[utoipa::path(
    description = "회원가입",
    post,
    path = "/signup",
    request_body = SignupInput,
    responses(
        (status = StatusCode::CREATED, description = "회원 가입 성공", body = UserResponse),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY, description = "이미 존재하는 이메일", body = ErrorBody, 
            example = json!({"error": "validation_error", "message": "이미 존재하는 이메일입니다".to_string()})
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR, description = "내부 서버 오류", body = ErrorBody,
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

#[utoipa::path(
    description = "로그인",
    post,
    path = "/login",
    request_body = LoginInput,
    responses(
        (status = 200, description = "로그인 성공", body = TokenResponse),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY, description = "이메일 또는 비밀번호가 올바르지 않습니다", body = ErrorBody, 
            example = json!({"error": "validation_error", "message": "이메일 또는 비밀번호가 올바르지 않습니다".to_string()})
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR, description = "내부 서버 오류", body = ErrorBody,
            example = json!({ "error": "internal_error", "message":  "서버 내부 오류가 발생했습니다" })
        )
    ),
    tags = ["auth"]
)]
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<TokenResponse>, AppError> {
    let user = state.user_service.login(input).await?;

    let token = jwt::create_token(
        &user,
        &state.configuration.jwt_secret,
        state.configuration.jwt_expiration_minutes,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("토큰 생성 실패: {e}")))?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer".to_string(),
    }))
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "auth", description = "인증 API")),
    paths(signup, login),
    components(schemas(UserResponse, SignupInput, ErrorBody, AppError))
)]
pub struct AuthOpenApiDoc;
