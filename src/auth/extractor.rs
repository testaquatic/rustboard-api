use axum::{extract::FromRequestParts, http::header};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::{auth::jwt::Claims, domain::role::Role, error::AppError, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub email: String,
    pub role: Role,
}

impl AuthUser {
    pub fn from_claims(claims: Claims) -> Result<Self, AppError> {
        let user_id = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;
        let role = claims.role.parse().map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser {
            user_id,
            email: claims.email,
            role,
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
        let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return Ok(OptionalAuthUser(None));
        };

        // Bearer 접두사가 없으면 None
        let Some(token) = auth_header.strip_prefix("Bearer ") else {
            return Ok(OptionalAuthUser(None));
        };

        // 토큰 디코딩 및 검증
        let token_data = decode(
            token,
            &DecodingKey::from_secret(state.configuration.jwt_secret.as_bytes()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        );

        match token_data {
            Ok(data) => {
                let auth_user = AuthUser::from_claims(data.claims)?;
                Ok(OptionalAuthUser(Some(auth_user)))
            }
            Err(e) => {
                tracing::debug!(error = %e, "선택적 인증: 토큰 검증 실패, 비인증으로 진행");
                Ok(OptionalAuthUser(None))
            }
        }
    }
}
