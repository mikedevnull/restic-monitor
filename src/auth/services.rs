use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use sqlx::SqlitePool;

use crate::{auth::repositories::UserRepository, error::RustMonitorError};

pub struct AuthService {
    user_repo: UserRepository,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        AuthService {
            user_repo: UserRepository::new(pool),
        }
    }

    pub async fn has_admin(&self) -> bool {
        self.user_repo.find_by_name("admin").await.is_some()
    }

    pub async fn add_user(&self, username: &str, password: &str) -> Result<(), RustMonitorError> {
        let hashed_password = Argon2::default().hash_password(password.as_bytes())?;
        let user_id = self
            .user_repo
            .create(username, &hashed_password.to_string())
            .await?;
        self.user_repo.get_by_id(user_id).await?;
        Ok(())
    }

    pub async fn verify_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(), argon2::password_hash::Error> {
        let user = self
            .user_repo
            .find_by_name(username)
            .await
            .ok_or(argon2::password_hash::Error::PasswordInvalid)?;
        Argon2::default().verify_password(password.as_bytes(), user.password.as_str())
    }
}
