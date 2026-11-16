use sqlx::SqlitePool;

#[derive(Debug, PartialEq, Eq, sqlx::Type)]
pub struct User {
    id: i64,
    pub username: String,
    pub password: String,
}

pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        UserRepository { pool }
    }

    pub async fn find_by_name(&self, username: &str) -> Option<User> {
        sqlx::query_as!(
            User,
            "SELECT id as \"id!\", username, password FROM users WHERE username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await
        .ok()?
    }
    pub async fn get_by_id(&self, id: i64) -> Result<User, sqlx::Error> {
        sqlx::query_as!(User, "SELECT * from users WHERE id==$1", id)
            .fetch_one(&self.pool)
            .await
    }
    pub async fn create(&self, username: &str, hashed_password: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO users (username, password) VALUES($1,$2)",
            username,
            hashed_password
        )
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}
