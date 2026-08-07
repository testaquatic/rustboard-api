use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::service::error::ServiceError;

#[derive(Error, Debug, ToSchema)]
pub enum AppError {
    #[error("{entity}(id={id})를 찾을 수 없습니다")]
    NotFound { entity: &'static str, id: i64 },

    #[error("입력값 검증 실패: {0}")]
    Validation(String),

    #[error("인증이 필요합니다")]
    Unauthorized,

    #[error("권한이 없습니다")]
    Forbidden,

    #[error("서버 내부 오류")]
    #[schema(value_type = ErrorBody)]
    Internal(#[source] anyhow::Error),
}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound { entity, id } => AppError::NotFound { entity, id },
            ServiceError::Validation(msg) => AppError::Validation(msg),
            ServiceError::Repo(repo_err) => AppError::Internal(repo_err.into()),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            // 로깅
            AppError::Internal(err) => {
                tracing::error!(
                    error.type = "internal",
                    error.message = %self,
                    error.detail = ?err,
                    "unhandled server error"
                )
            }
            AppError::Unauthorized | AppError::Forbidden => {
                tracing::warn!(
                    error.type = "auth",
                    error.message = %self,
                    "authentication/authorization failure"
                )
            }
            AppError::NotFound { .. } => {
                tracing::debug!(
                    error.type = "not_found",
                    error.message = %self,
                    "resource found error"
                )
            }
            AppError::Validation(_) => {
                tracing::debug!(
                    error.type = "validation",
                    error.message = %self,
                    "input validation failure"
                )
            }
        }

        // HTTP 응답 생성
        let (status, error_code, message) = match self {
            AppError::NotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("{entity}(id={id})를 찾을 수 없습니다").to_string(),
            ),
            AppError::Validation(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", msg)
            }
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "인증이 필요합니다".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "권한이 없습니다".to_string(),
            ),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "서버 내부 오류가 발생했습니다".to_string(),
            ),
        };

        let body = ErrorBody {
            error: error_code,
            message,
        };

        (status, Json(body)).into_response()
    }
}
