use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::domain::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 사용자 iD
    pub sub: String,
    pub email: String,
    pub role: String,
    /// 만료시간
    /// unix타임스탬프
    pub exp: i64,
    pub username: String,
}

pub fn create_token(
    user: &User,
    secret: &str,
    expiration_minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::minutes(expiration_minutes)).timestamp();

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: "user".to_string(),
        exp,
        username: user.display_name.clone(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
