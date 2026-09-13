use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

/// 평문 비밀번호를 argon2id로 해싱
/// 반환값은 PHC 문자열
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(hash.to_string())
}

/// 저장된 해시와 입력 비밀번호가 일치하는지 확인
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);

    Ok(result.is_ok())
}
