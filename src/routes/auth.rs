use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use crate::{
    auth::{extractor::AuthUser, jwt},
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

#[utoipa::path(
    description = "현재 사용자 정보 조회",
    get,
    path = "/me",
    security(("AuthUser" = ["read:items"])),
    responses(
        (
            status = StatusCode::OK,
            description = "현재 사용자 정보 조회 성공",
            body = Map<String, Value>,
            example = json!({"user_id": 1, "email": "[EMAIL_ADDRESS]", "role": "admin"})
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "인증이 필요",
            body = ErrorBody,
            example = json!({ "error": "unauthorized", "message": "인증이 필요합니다" })
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            description = "내부 서버 오류",
            body = ErrorBody,
            example = json!({ "error": "internal_error", "message":  "서버 내부 오류가 발생했습니다" })
        )
    ),
    tags = ["auth"]
)]
pub async fn me(auth_user: AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "user_id": auth_user.user_id,
        "email": auth_user.email,
        "role": auth_user.role.to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct AuthUserSecurity;

impl Modify for AuthUserSecurity {
    fn modify(&self, open_api: &mut utoipa::openapi::OpenApi) {
        if let Some(schema) = open_api.components.as_mut() {
            schema.add_security_scheme(
                "AuthUser",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "auth", description = "인증 API")),
    paths(signup, login, me),
    components(schemas(UserResponse, SignupInput, ErrorBody, AppError)),
    modifiers(&AuthUserSecurity),
)]
pub struct AuthOpenApiDoc;
