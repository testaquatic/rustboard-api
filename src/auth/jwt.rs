use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::domain::user::User;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String, // 사용자 ID
    pub email: String,
    pub role: String,
    pub exp: i64, // 만료 시간(UNIX Timestamp)
}

pub fn create_token(
    user: &User,
    secret: &str,
    expiration_minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + chrono::Duration::minutes(expiration_minutes);

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.to_string(),
        exp: expiration.timestamp(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}
