use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use secrecy::SecretString;

use crate::domain::user::User;
use crate::{auth::jwt::create_token, domain::user::LoginInput, error::AppError, state::AppState};

/// 회원 가입할 때 입력하는 정보
#[derive(Debug, serde::Deserialize)]
pub struct SignupInput {
    pub email: String,
    pub password: SecretString,
    pub display_name: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SignupResponse {
    pub id: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<User> for SignupResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.timestamp(),
            updated_at: value.updated_at.timestamp(),
        }
    }
}

pub async fn signup(
    app_state: State<AppState>,
    signup_input: Json<SignupInput>,
) -> Result<(StatusCode, Json<SignupResponse>), AppError> {
    let user = app_state.user_service.signup(&signup_input).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
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
