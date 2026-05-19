use std::fmt::Debug;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::service::error::ServiceError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{entity}(id={id})을 찾을 수 없습니다.")]
    NotFound { entity: String, id: i64 },
    #[error("입력값 검증 실패: {0}")]
    Validation(String),
    #[error("인증이 필요합니다")]
    Unauthorized,
    #[error("권한이 없습니다")]
    Forbiddn,
    #[error("서버 내부 오류")]
    Internal(#[source] anyhow::Error),
    // #[error("이미 존재하는 리소스입니다")]
    // Conflict,
    #[error("동시 연결 수 초과")]
    TooMayConnections,
}

impl AppError {
    fn safe_log_message(&self) -> String {
        match self {
            AppError::Internal(err) => {
                let msg = format!("{err:?}");
                mask_sensitive(&msg)
            }
            _ => {
                format!("{self}")
            }
        }
    }
}

/// 알려진 패턴을 마스킹한다.
/// DB주소, 사설망 IP주소 ,Email 주소
fn mask_sensitive(input: &str) -> String {
    let mut result = input.to_string();

    // DB 접속 패턴
    let db_url_re = regex::Regex::new(r#"postgres://[^@]+@[^\s/]+"#).unwrap();
    result = db_url_re
        .replace_all(&result, "[MASKED_POSTGRES]")
        .to_string();

    // IP 포트 패턴
    // 구글 검색을 해보니 사설망은 이렇다.
    // 10.0.0.0 ~ 10.255.255.255 (10.0.0.0/8)
    // 172.16.0.0 ~ 172.31.255.255 (172.16.0.0/12)
    // 192.168.0.0 ~ 192.168.255.255 (192.168.0.0/16)
    let ip_re = regex::Regex::new(
        r#"\b(?:10\.\d{1,3}|172\.(?:1[6-9]|2[0-9]|3[0-1])|192\.168)\.\d{1,3}\.\d{1,3}(?::\d+)?\b"#,
    )
    .unwrap();
    result = ip_re.replace_all(&result, "[MASKED_IP]").to_string();

    // 이메일 패턴
    let email_re = regex::Regex::new(r#"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"#).unwrap();
    result = email_re.replace_all(&result, "[MASKED_EMAIL]").to_string();

    result
}

/// 디버깅할 때는 마스킹을 비활성화한다.
/// 이를 위한 헬퍼 함수이다.
fn should_mask() -> bool {
    std::env::var("RUST_ENV")
        .map(|v| v != "development" && v != "test")
        .unwrap_or(true)
}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound { entity, id } => AppError::NotFound { entity, id },
            ServiceError::Validation(msg) => AppError::Validation(msg),
            ServiceError::Repo(err) => AppError::Internal(err.into()),
            ServiceError::PasswordHash(msg) => AppError::Internal(anyhow::anyhow!(msg)),
            ServiceError::Forbidden => AppError::Forbiddn,
        }
    }
}

/// 오류시 응답
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 로깅
        match &self {
            // 500, 즉시 확인 필요
            AppError::Internal(err) => {
                let detail_message = if should_mask() {
                    self.safe_log_message()
                } else {
                    format!("{err:}")
                };

                tracing::error!(
                    error.type = "internal",
                    error.message = %self,
                    error.detail = ?detail_message,
                    "unhandled server error"
                );
            }

            // 401, 403 보안 관련
            AppError::Unauthorized | AppError::Forbiddn => tracing::warn!(
                error.type = "auth",
                error.message = %self,
                "authentication/authorization failure"
            ),

            // 404
            AppError::NotFound { .. } => tracing::debug!(
                error.type = "not_found",
                error.message = %self,
                "resource not found",
            ),

            // 422
            AppError::Validation(_) => tracing::debug!(
              error.type = "validation",
              error.message = %self,
              "input validation failed"
            ),

            AppError::TooMayConnections => tracing::warn!(
                error.type = "too_many_connections",
                error.message = %self,
                "too many connections"
            ),
        }

        // 응답
        let (status, error_code, message) = match self {
            // 404
            AppError::NotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("{entity}(id={id})을 찾을 수 없습니다"),
            ),
            // 422
            AppError::Validation(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", msg)
            }
            // 401
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "인증이 필요합니다".to_string(),
            ),
            // 403
            AppError::Forbiddn => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "권한이 없습니다".to_string(),
            ),
            // 500
            // 세부적인 내용은 숨겨야 한다
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "서버 내부 오류가 발생했습니다".to_string(),
            ),
            AppError::TooMayConnections => (
                StatusCode::SERVICE_UNAVAILABLE,
                "too_many_connections",
                "서버 연결 수 초과, 잠수 후 재시도해주세요.".to_string(),
            ),
        };

        crate::metrics::increment_error_counter("error_code");

        let body = ErrorBody {
            error: error_code,
            message,
        };

        (status, Json(body)).into_response()
    }
}
