#[derive(thiserror::Error, Debug)]
pub enum RepositoryError {
    #[error("데이터베이스 쿼리 오류")]
    Query(#[from] sqlx::Error),

    #[error("{entity}(id={id})를 찾을 수 없습니다")]
    NotFound { entity: String, id: i64 },

    #[error("이미 존재하는 {entity} 입니다.")]
    AlreadyExists { entity: String },
}
