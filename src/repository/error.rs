use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("데이터베이스 쿼리 오류")]
    Query(#[from] sqlx::Error),

    #[error("{entity}(id={id})를 찾을 수 없습니다")]
    NotFound { entity: &'static str, id: i64 },
}
