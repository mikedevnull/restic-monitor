use secrecy::SecretString;
use sqlx::SqlitePool;

use crate::repo::repositories::RepoRepository;

pub struct RepoService {
    repo: RepoRepository,
}

impl RepoService {
    fn new(db: SqlitePool, secret_app_key: &SecretString) -> Self {
        Self {
            repo: RepoRepository::new(db, secret_app_key),
        }
    }
}
