#[cfg(test)]
use crate::{init_routes, setup_db_with_migrations, setup_tera};
#[cfg(test)]
use actix_web::{App, web};
#[cfg(test)]
use sea_orm::DatabaseConnection;

#[cfg(test)]
pub async fn setup_test_app() -> (axum_test::TestServer, DatabaseConnection) {
    let db = setup_db_with_migrations("sqlite::memory:").await;
    let tera = setup_tera();

    let db_handle_for_testing = db.clone();

    (
        axum_test::TestServer::new(move || {
            App::new()
                .app_data(web::Data::new(tera.clone()))
                .app_data(web::Data::new(db.clone()))
                .configure(init_routes)
        }),
        db_handle_for_testing,
    )
}
