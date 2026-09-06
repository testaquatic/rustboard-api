use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::service::error::ServiceError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{entity} (id={id})을 찾을 수 없습니다")]
    NotFound { entity: String, id: i64 },

    #[error("입력값 검증 실패: {0}")]
    Validation(String),

    #[error("인증이 필요합니다")]
    Unauthorized,

    #[error("권한이 없습니다")]
    Forbidden,

    #[error("이미 존재하는 데이터입니다")]
    Conflict(String),

    #[error("비밀번호 처리 오류")]
    PasswordHash(String),

    #[error("내부 서버 오류")]
    Internal(anyhow::Error),
}

#[derive(serde::Serialize)]
pub struct AppErrorResponse {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type, message) = match &self {
            AppError::NotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("{entity}(id={id})을 찾을 수 없습니다"),
            ),
            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                msg.clone(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "인증이 필요합니다".into(),
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "권한이 없습니다".into()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::PasswordHash(_) | AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_server_error",
                "서버 내부 오류가 발생했습니다".into(),
            ),
        };

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = ?self, "Internal server error");
        }

        (
            status,
            Json(AppErrorResponse {
                error: error_type,
                message,
            }),
        )
            .into_response()
    }
}

impl From<ServiceError> for AppError {
    fn from(service_error: ServiceError) -> Self {
        match service_error {
            ServiceError::Repo(err) => AppError::Internal(err.into()),
            ServiceError::Validation(msg) => AppError::Validation(msg),
            ServiceError::NotFound { entity, id } => AppError::NotFound { entity, id },
            ServiceError::PasswordHash(msg) => AppError::PasswordHash(msg),
            ServiceError::Forbidden => AppError::Forbidden,
        }
    }
}
