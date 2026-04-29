use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub service_name: String,
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

        Ok(Self {
            bind_addr,
            service_name,
        })
    }
}
