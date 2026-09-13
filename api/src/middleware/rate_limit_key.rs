use tower_governor::{GovernorError, key_extractor::KeyExtractor};

#[derive(Clone)]
pub struct ForwardedIpKeyExtractor;

impl KeyExtractor for ForwardedIpKeyExtractor {
    type Key = String;

    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        if let Some(forwarded) = req.headers().get("x-forwarded-for")
            && let Ok(value) = forwarded.to_str()
            && let Some(first_ip) = value.split(",").next()
        {
            return Ok(first_ip.trim().to_string());
        }

        req.extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .ok_or(GovernorError::UnableToExtractKey)
    }
}
