use crate::{
    auth::password,
    domain::user::{SignupInput, User},
    repository::user::DynUserRepository,
    service::error::ServiceError,
};

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

        // 패스워드 해싱
        let password_hash = password::hash_password(&input.password)
            .map_err(|e| ServiceError::PasswordHash(e.to_string()))?;

        // DB에 저장
        let user = self
            .repo
            .insert(&input.email, &password_hash, &input.display_name)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ServiceError> {
        let user = self.repo.find_by_email(email).await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<User, ServiceError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound { entity: "user", id })
    }
}
