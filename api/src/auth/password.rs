use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash};
use password_hash::{SaltString, rand_core::OsRng};

/// 평문 비밀번호를 argon2id로 해싱한다.
/// 반환값은 PHC 문자열이다.
/// https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

/// 저장된 해시와 입력 비밀번호가 일치하는지 검증한다.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(is_valid)
}
