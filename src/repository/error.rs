/// 저장소 계층에서 반환하는 오류
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("데이터베이스 쿼리 오류")]
    Query(#[from] sqlx::Error),
    #[error("{entity}(id={id})을 찾을 수 없습니다")]
    NotFound { entity: &'static str, id: i64 },
}
