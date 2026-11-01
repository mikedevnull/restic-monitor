use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustMonitorError {
    #[error("Database related error")]
    Database(#[from] sea_orm::DbErr),
}

impl actix_web::ResponseError for RustMonitorError {
    fn status_code(&self) -> actix_http::StatusCode {
        actix_http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
