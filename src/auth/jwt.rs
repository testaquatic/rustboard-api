use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, SecretString};

use crate::domain::user::User;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
}

pub fn create_token(
    user: &User,
    secret: &SecretString,
    expiration_minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::minutes(expiration_minutes)).timestamp();
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role.clone(),
        exp,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
    )
}

pub fn verify_token(
    token: &str,
    secret: &SecretString,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}
