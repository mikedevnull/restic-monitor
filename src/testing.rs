#[cfg(test)]
use crate::{auth::AuthService, init_routes, setup_tera};
#[cfg(test)]
use actix_web::{App, web};

#[cfg(test)]
pub async fn setup_test_app(db: sqlx::SqlitePool) -> axum_test::TestServer {
    crate::db::migrate(db.clone())
        .await
        .expect("Database migration failed");
    let tera = setup_tera();

    axum_test::TestServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(AuthService::new(db.clone())))
            .configure(init_routes)
    })
}
