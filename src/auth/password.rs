use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};

/// 평문 비밀번호를 argon2id로 해싱한다.
pub fn hash_password(
    password: &SecretString,
) -> Result<SecretString, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.expose_secret().as_bytes())?;

    Ok(SecretString::new(hash.to_string().into()))
}

/// 저장된 해시와 입력 비밀번호가 일치하는지 검증한다
pub fn verify_password(
    password: &SecretString,
    hash: &SecretString,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash.expose_secret())?;

    Ok(Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
        .is_ok())
}

/// 해시가 bcrypt 형식인지 확인한다
pub fn is_bcrypt_hash(hash: &SecretString) -> bool {
    hash.expose_secret().starts_with("$2b$")
        || hash.expose_secret().starts_with("$2a$")
        || hash.expose_secret().starts_with("$2y$")
}

/// bcrypt 해시를 검증한다
pub fn verify_bcrypt(
    password: &SecretString,
    hash: &SecretString,
) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password.expose_secret(), hash.expose_secret())
}

/// 로그인시 호출:
///
/// 해시 포맷을 감지하고 bcrypt이면 검증 후 재해싱 필요 여부를 반환한다
pub struct VerifyResult {
    pub is_valid: bool,
    pub needs_rehash: bool,
    pub new_hash: Option<SecretString>,
}

pub fn verify_and_maybe_rehash(
    password: &SecretString,
    stored_hash: &SecretString,
) -> Result<VerifyResult, String> {
    if is_bcrypt_hash(stored_hash) {
        let is_valid = verify_bcrypt(password, stored_hash).map_err(|e| e.to_string())?;

        if is_valid {
            let new_hash = hash_password(password).map_err(|e| e.to_string())?;

            Ok(VerifyResult {
                is_valid: true,
                needs_rehash: true,
                new_hash: Some(new_hash),
            })
        } else {
            Ok(VerifyResult {
                is_valid: false,
                needs_rehash: false,
                new_hash: None,
            })
        }
    } else {
        let is_valid = verify_password(password, stored_hash).map_err(|e| e.to_string())?;
        Ok(VerifyResult {
            is_valid,
            needs_rehash: false,
            new_hash: None,
        })
    }
}
