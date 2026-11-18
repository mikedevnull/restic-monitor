use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use argon2::{Argon2, password_hash::phc::Error::Base64};
use base64::Engine;
use rand::{TryRng, rngs::SysRng};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Database, Encode, Sqlite, SqlitePool, encode::IsNull, error::BoxDynError};

pub struct S3BackendOptions {}
pub struct LocalBackendOptions {}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum BackendOptions {
    LocalBackendOptions,
    S3BackendOptions,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ResticRepo {
    id: i64,
    name: String,
    password: SecretString,
    #[sqlx(json)]
    backend: BackendOptions,
}

impl sqlx::Type<Sqlite> for EncryptBind<'_> {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

pub struct EncryptBind<'a> {
    value: &'a str,
    cipher: &'a Aes256Gcm,
}

impl<'q> Encode<'q, Sqlite> for EncryptBind<'_> {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        let mut nonce_bytes = [0u8; 12];
        SysRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| std::io::Error::other(format!("Nonce generation failed: {e}")))?;
        let nonce = Nonce::from(nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, self.value.as_bytes())
            .map_err(|e| format!("Encryption failed: {e}"))?;

        let mut payload = Vec::with_capacity(12 + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);

        let b64_payload = base64::engine::general_purpose::STANDARD.encode(payload);

        <String as Encode<'q, Sqlite>>::encode(b64_payload, buf)
    }
}

pub struct RepoRepository {
    db: SqlitePool,
    cipher: Aes256Gcm,
}

impl RepoRepository {
    pub fn new(db: SqlitePool, secret_app_key: &SecretString) -> Self {
        let mut output_key_material = [0u8; 32];
        Argon2::default()
            .hash_password_into(
                secret_app_key.expose_secret().as_bytes(),
                "uselesssalt".as_bytes(),
                &mut output_key_material,
            )
            .expect("Derive key from app secret");
        let cipher = Aes256Gcm::new_from_slice(&output_key_material).unwrap();
        Self { db, cipher }
    }

    pub async fn create(
        &self,
        name: &str,
        password: &SecretString,
        backend: &BackendOptions,
    ) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            "INSERT INTO repositories (name, password, backend_options) VALUES ($1,$2,$3)",
            name,
            EncryptBind {
                value: password.expose_secret(),
                cipher: &self.cipher
            },
            sqlx::types::Json(backend)
        )
        .execute(&self.db)
        .await?
        .last_insert_rowid();
        Ok(row)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<ResticRepo, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, name, password, backend_options as \"backend_options: String\" FROM repositories WHERE id = $1",
            id
        )
        .fetch_one(&self.db)
        .await?;

        let payload = base64::engine::general_purpose::STANDARD
            .decode(row.password)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        if payload.len() < 12 {
            return Err(sqlx::Error::Decode(Box::new(std::io::Error::other(
                "Encrypted password payload is too short",
            ))));
        }

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let nonce: [u8; 12] = nonce_bytes.try_into().map_err(|_| {
            sqlx::Error::Decode(Box::new(std::io::Error::other(
                "Encrypted password nonce has invalid length",
            )))
        })?;
        let nonce = Nonce::from(nonce);
        let plaintext = self.cipher.decrypt(&nonce, ciphertext).map_err(|error| {
            sqlx::Error::Decode(Box::new(std::io::Error::other(format!(
                "Password decryption failed: {error}"
            ))))
        })?;
        let password =
            String::from_utf8(plaintext).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        let backend = serde_json::from_str(&row.backend_options)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;

        Ok(ResticRepo {
            id: row.id,
            name: row.name,
            password: SecretString::from(password),
            backend,
        })
    }
}

#[cfg(test)]
mod test {

    use crate::repo::repositories::BackendOptions::LocalBackendOptions;

    use super::RepoRepository;
    use secrecy::{ExposeSecret, SecretString};
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn stores_encrypted_repo_data(pool: SqlitePool) -> Result<(), sqlx::Error> {
        let repo = RepoRepository::new(pool.clone(), &SecretString::from("fancyappkey"));
        let password = SecretString::from("supersecret".to_owned());
        let new_repo_id = repo
            .create("dummy", &password, &LocalBackendOptions)
            .await
            .unwrap();

        let stored_password = sqlx::query!(
            "SELECT password FROM repositories WHERE id = $1",
            &new_repo_id
        )
        .fetch_one(&pool)
        .await?;

        assert_ne!(stored_password.password, "supersecret");
        Ok(())
    }

    #[sqlx::test]
    async fn can_decode_data_with_correct_key(pool: SqlitePool) -> Result<(), sqlx::Error> {
        let new_repo_id = {
            let repo = RepoRepository::new(pool.clone(), &SecretString::from("fancyappkey"));
            let password = SecretString::from("supersecret".to_owned());
            repo.create("dummy", &password, &LocalBackendOptions)
                .await
                .unwrap()
        };

        let repo = RepoRepository::new(pool.clone(), &SecretString::from("fancyappkey"));
        let repository = repo.get_by_id(new_repo_id).await?;
        assert_eq!(repository.name, "dummy");
        assert_eq!(repository.password.expose_secret(), "supersecret");

        Ok(())
    }

    #[sqlx::test]
    async fn cannot_decode_data_with_incorrect_key(pool: SqlitePool) -> Result<(), sqlx::Error> {
        let new_repo_id = {
            let repo = RepoRepository::new(pool.clone(), &SecretString::from("fancyappkey"));
            let password = SecretString::from("supersecret".to_owned());
            repo.create("dummy", &password, &LocalBackendOptions)
                .await
                .unwrap()
        };

        let repo = RepoRepository::new(pool.clone(), &SecretString::from("thisisnotthecorrectkey"));
        repo.get_by_id(new_repo_id)
            .await
            .expect_err("Fails to decode with wrong key");

        Ok(())
    }
}
