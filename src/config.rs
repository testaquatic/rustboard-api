use std::net::SocketAddr;

/// 설정
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub service_name: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("필수 환경 변수가 없습니다: {0}")]
    Missing(&'static str),
    #[error("환경 변수 {0}의 형식이 올바르지 않습니다: {1}")]
    Invalid(&'static str, String),
    #[error("환경 변수 {0}을 파싱할 수 없습니다: {1}")]
    Parse(&'static str, String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // BIND_ADDR
        // 기본값은 "0.0.0.0:3000"
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        let bind_addr = bind_addr
            .parse::<SocketAddr>()
            .map_err(|e| ConfigError::Invalid("BIND_ADDR", e.to_string()))?;

        // SERVICE_NAME
        // 환경변수가 없으면 "rustboard-api"
        let service_name =
            std::env::var("SERVICE_NAME").unwrap_or_else(|_| "rustboard-api".to_string());

        // DATABASE_URL
        // 환경변수가 없으면 오류
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

        // JWT_SECRET
        // 환경변수가 없으면 오류
        let jwt_secret =
            std::env::var("JWT_SECRET").map_err(|_| ConfigError::Missing("JWT_SECRET"))?;

        // JWT_EXPIRATION_MINUTES
        // 기본값은 15
        let jwt_expiration = std::env::var("JWT_EXPIRATION)JWT_EXPIRATION_MINUTES")
            .unwrap_or_else(|_| "15".to_string())
            .parse::<i64>()
            .map_err(|e| ConfigError::Parse("JWT_EXPIRATION_MINUTES", e.to_string()))?;

        Ok(Self {
            bind_addr,
            service_name,
            database_url,
            jwt_secret,
            jwt_expiration_minutes: jwt_expiration,
        })
    }
}
