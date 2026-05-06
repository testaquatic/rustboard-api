use argon2::{Argon2, PasswordHasher};
use password_hash::{SaltString, rand_core::OsRng};

/// 평문 비밀번호를 argon2id로 해싱한다.
/// 반환값은 PHC 문자열이다.
/// https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md
pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}
