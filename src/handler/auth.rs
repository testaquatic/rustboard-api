use axum::{Json, extract::State, http::StatusCode};

use crate::{
    auth::jwt,
    domain::user::{LoginInput, SignupInput, TokenResponse, UserResponse},
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

/// 로그인 핸들러
#[utoipa::path(
    post,
    tag = "auth",
    path = "/login",
    summary = "로그인",
    description = "로그인 엔드포인트",
    request_body = LoginInput,
    responses((
        status = StatusCode::OK,
        content_type = "application/json",
        body = UserResponse
    ),(
        status = StatusCode::INTERNAL_SERVER_ERROR,
        description = "토큰 생성 실패",
        content_type = "application/json",
        body = ErrorBody,
    ))
)]
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<TokenResponse>, AppError> {
    let user = state.users_service.login(input).await?;

    let token = jwt::create_token(
        &user,
        &state.config.jwt_secret,
        state.config.jwt_expiration_minutes,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("토큰 생성 실패: {e}")))?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer".to_string(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
pub struct AuthOpenApi;
