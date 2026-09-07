use secrecy::SecretString;
use serde::Deserialize;

/// 서버의 설정
#[derive(Deserialize, Clone)]
pub struct Settings {
    /// 서버의 주소
    pub app_addr: String,
    /// 데이터베이스의 URL
    pub database_url: String,
    /// OpenTelemetry OTLP Export Endpoint
    pub otel_exporter_otlp_endpoint: String,
    /// JWT Secret
    pub jwt_secret: SecretString,
    /// JWT 토큰 만료 시간(분)
    pub jwt_token_expiration_minutes: i64,
}

impl Settings {
    /// 설정을 로드한다.
    /// 순서
    /// 1. 설정 파일 (configuration/base.yaml)
    /// 2. 환경 변수 (APP_APP_ADDR, APP_DATABASE_URL)
    pub fn build() -> Result<Settings, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("configuration/base.yaml"))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}
