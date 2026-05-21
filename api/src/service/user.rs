use std::sync::Arc;

use rustboard_domain::user::{LoginInput, SignupInput, User};

use crate::{auth::password, repository::types::UserRepository, service::error::ServiceError};

type DynUserRepository = Arc<dyn UserRepository + Send + Sync>;

pub struct UserService {
    repo: DynUserRepository,
}

impl UserService {
    pub fn new(repo: DynUserRepository) -> Self {
        Self { repo }
    }

    pub async fn signup(&self, input: SignupInput) -> Result<User, ServiceError> {
        // 이메일 중복 검사
        if self.repo.find_by_email(&input.email).await?.is_some() {
            return Err(ServiceError::Validation(
                "이미 사용 중인 이메일입니다".to_string(),
            ));
        }

        // argon2id 해싱
        let password_hash = password::hash_password(&input.password)
            .map_err(|e| ServiceError::PasswordHash(e.to_string()))?;

        self.repo
            .insert(&input.email, &password_hash, &input.display_name)
            .await
            .map_err(From::from)
    }

    pub async fn login(&self, input: LoginInput) -> Result<User, ServiceError> {
        let user = self
            .repo
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| {
                ServiceError::Validation("이메일 또는 비밀번호가 올바르지 않습니다".to_string())
            })?;

        let is_valid = password::verify_password(&input.password, &user.password_hash)
            .map_err(|e| ServiceError::PasswordHash(e.to_string()))?;

        if !is_valid {
            return Err(ServiceError::Validation(
                "이메일 또는 비밀번호가 올바르지 않습니다".to_string(),
            ));
        }

        Ok(user)
    }
}
