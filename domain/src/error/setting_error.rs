use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("IO 오류: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config 라이브러리 오류: {0}")]
    Config(#[from] config::ConfigError),
}
