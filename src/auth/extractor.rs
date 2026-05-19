use std::str::FromStr;

use axum::{extract::FromRequestParts, http::header::AUTHORIZATION};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::{auth::jwt::Claims, domain::role::Role, error::AppError, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub email: String,
    pub role: Role,
    pub name: String,
}

impl AuthUser {
    pub fn from_claims(claims: Claims) -> Result<Self, AppError> {
        let Ok(user_id) = claims.sub.parse() else {
            return Err(AppError::Unauthorized);
        };

        let role = Role::from_str(&claims.role).map_err(|_| AppError::Unauthorized)?;

        Ok(Self {
            user_id,
            email: claims.email,
            role,
            name: claims.username,
        })
    }

    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
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
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Bearer 접두사 제거
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // 토큰 검증
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|e| {
            tracing::warn!(error = %e, error_kind = ?e.kind(), "JWT 검증 실패");
            AppError::Unauthorized
        })?;

        AuthUser::from_claims(token_data.claims)
    }
}

#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Authorization 헤더가 없으면 None
        let Some(token) = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return Ok(Self(None));
        };

        // Bearer 접두사가 없으면 None
        let Some(token) = token.strip_prefix("Bearer ") else {
            return Ok(Self(None));
        };

        // 토큰을 검증시도하고 실패하면 None
        decode(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map(|data| {
            let auth_user = AuthUser::from_claims(data.claims)?;
            Ok(Self(Some(auth_user)))
        })
        .unwrap_or_else(|e| {
            tracing::debug!(error = %e, "선택적 인증: 토큰 검증 실패, 비인증으로 진행");
            Ok(Self(None))
        })
    }
}
