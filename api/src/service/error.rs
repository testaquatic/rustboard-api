use rustboard_domain::error::RepositoryError;

use crate::error::AppError;

/// 서비스 계층에서 반환하는 오류
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("리포지토리 오류")]
    Repo(#[from] RepositoryError),

    #[error("입력값 검증 실패: {0}")]
    Validation(String),

    #[error("{entity}(id={id})을 찾을 수 없습니다")]
    NotFound { entity: String, id: i64 },

    #[error("비밀번호 처리 오류")]
    PasswordHash(String),

    #[error("권한이 없습니다")]
    Forbidden,
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
