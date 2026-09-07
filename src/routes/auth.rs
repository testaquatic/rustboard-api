use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use secrecy::SecretString;

use crate::{auth::jwt::create_token, domain::user::LoginInput, error::AppError, state::AppState};

/// 회원 가입할 때 입력하는 정보
#[derive(Debug, serde::Deserialize)]
pub struct SignupInput {
    pub email: String,
    pub password: SecretString,
    pub display_name: String,
}

pub async fn signup(
    app_state: State<AppState>,
    signup_input: Json<SignupInput>,
) -> Result<axum::http::StatusCode, AppError> {
    app_state.user_service.signup(&signup_input).await?;

    Ok(axum::http::StatusCode::CREATED)
}

/// 로그인 했을 때 돌려주는 정보
#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    app_state: State<AppState>,
    login_input: Json<LoginInput>,
) -> Result<Response, AppError> {
    let user = app_state.user_service.login(login_input.0).await?;
    let token = create_token(
        &user,
        &app_state.configuration.jwt_secret,
        app_state.configuration.jwt_token_expiration_minutes,
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok((StatusCode::OK, Json(LoginResponse { token })).into_response())
}
