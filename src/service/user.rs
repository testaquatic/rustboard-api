use crate::{
    auth::password::{self},
    domain::user::{LoginInput, User},
    repository::user::UserRepository,
    routes::auth::SignupInput,
    service::error::ServiceError,
};

pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn login(&self, input: LoginInput) -> Result<User, ServiceError> {
        let user = self
            .user_repo
            .find_by_email(&input.email)
            .await?
            .ok_or_else(|| {
                ServiceError::Validation("이메일 또는 비밀번호가 올바르지 않습니다".to_string())
            })?;

        let password_clone = input.password.clone();
        let hash_clone = user.password_hash.clone();
        let result = tokio::task::spawn_blocking(move || {
            password::verify_and_maybe_rehash(&password_clone, &hash_clone)
        })
        .await
        .map_err(|e| ServiceError::PasswordHash(e.to_string()))?
        .map_err(ServiceError::PasswordHash)?;

        if result.needs_rehash
            && let Some(new_hash) = result.new_hash
        {
            tracing::info!(user_id = user.id, "bcrypt -> argon2 해시 이관");
            metrics::counter!("auth.hash_migration", "from" => "bcrypt", "to" => "argon2")
                .increment(1);

            self.user_repo
                .update_password_hash(user.id, &new_hash)
                .await?;
        }

        Ok(user)
    }

    pub async fn signup(&self, signup_input: &SignupInput) -> Result<User, ServiceError> {
        let signup_input_password_clone = signup_input.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || {
            password::hash_password(&signup_input_password_clone)
        })
        .await
        .map_err(|e| ServiceError::PasswordHash(e.to_string()))?
        .map_err(|e| ServiceError::PasswordHash(e.to_string()))?;

        let user = self
            .user_repo
            .add_user(
                &signup_input.email,
                &password_hash,
                &signup_input.display_name,
                "user",
            )
            .await?;

        Ok(user)
    }
}
