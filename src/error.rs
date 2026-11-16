use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustMonitorError {
    #[error("Database related error")]
    Database(#[from] sqlx::Error),
    #[error("Auth related error")]
    AuthError(#[from] argon2::password_hash::Error),
}

impl actix_web::ResponseError for RustMonitorError {
    fn status_code(&self) -> actix_http::StatusCode {
        actix_http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
