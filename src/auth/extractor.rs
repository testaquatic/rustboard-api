use axum::{extract::FromRequestParts, http::header};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::{auth::jwt::Claims, error::AppError, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub email: String,
    pub role: String,
}

impl AuthUser {
    pub fn from_claims(claims: Claims) -> Result<Self, AppError> {
        let user_id = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser {
            user_id,
            email: claims.email,
            role: claims.role,
        })
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Authorization 헤더 꺼내기
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Bearer 접두사 제거
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // 토큰 디코딩 및 검증
        let token_data = decode(
            token,
            &DecodingKey::from_secret(state.configuration.jwt_secret.as_bytes()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|_| AppError::Unauthorized)?;

        AuthUser::from_claims(token_data.claims)
    }
}
