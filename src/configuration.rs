use std::net::SocketAddr;

use config::Config;
use serde::Deserialize;
use thiserror::Error;

/// 설정
#[derive(Deserialize)]
pub struct Settings {
    pub bind_addr: SocketAddr,
    pub service_name: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_minutes: i64,
}

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("IO 오류: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config 라이브러리 오류: {0}")]
    Config(#[from] config::ConfigError),
}

pub fn get_configuration() -> Result<Settings, SettingsError> {
    let base_path = std::env::current_dir()?;
    let config_path = base_path.join("configuration");

    Config::builder()
        // 일단 기본 설정을 읽는다
        .add_source(config::File::from(config_path.join("base.yaml")))
        // 그 다음 환경 변수를 읽는다.
        // eg `APP_BIND_ADDR=0.0.0.0:3000`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize()
        .map_err(SettingsError::Config)
}
