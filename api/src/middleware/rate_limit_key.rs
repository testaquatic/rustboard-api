use tower_governor::key_extractor::KeyExtractor;

#[derive(Clone)]
pub struct ForwardedIpKeyExtractor;

impl KeyExtractor for ForwardedIpKeyExtractor {
    type Key = String;

    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        // X-Forwarded-For 헤더가 있으면 첫 번째 IP를 사용
        if let Some(forwarded) = req.headers().get("x-forwarded-for")
            && let Ok(value) = forwarded.to_str()
            && let Some(first_ip) = value.split(',').next()
        {
            return Ok(first_ip.trim().to_string());
        }

        // X-Forwared-For 헤더가 없으면 클라이언트 IP를 사용
        req.extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|cl| cl.0.ip().to_string())
            .ok_or(tower_governor::GovernorError::UnableToExtractKey)
    }
}
