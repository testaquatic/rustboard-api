use std::net::SocketAddr;

/// 설정
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub service_name: String,
    pub database_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("필수 환경 변수가 없습니다: {0}")]
    Missing(&'static str),
    #[error("환경 변수 {0}의 형식이 올바르지 않습니다: {1}")]
    Invalid(&'static str, String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        let bind_addr = bind_addr
            .parse::<SocketAddr>()
            .map_err(|e| ConfigError::Invalid("BIND_ADDR", e.to_string()))?;

        let service_name =
            std::env::var("SERVICE_NAME").unwrap_or_else(|_| "rustboard-api".to_string());
        // DATABASE 환경변수가 없으면 오류
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

        Ok(Self {
            bind_addr,
            service_name,
            database_url,
        })
    }
}
