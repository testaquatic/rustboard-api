use axum::{
    extract::{FromRef, FromRequestParts},
    http::header,
};

use crate::{auth::jwt::verify_token, error::AppError, state::AppState};

pub struct AuthUser {
    pub user_id: i64,
    pub email: String,
    pub role: String,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let app_state = AppState::from_ref(state);

        let claims = verify_token(token, &app_state.configuration.jwt_secret)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser {
            user_id: claims.sub.parse().map_err(|_| AppError::Unauthorized)?,
            email: claims.email.clone(),
            role: claims.role.clone(),
        })
    }
}
