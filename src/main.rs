use actix_files as fs;

use actix_web::{App, HttpServer, middleware::Logger, web};

use restic_monitor::{init_routes, setup_db_with_migrations, setup_tera};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let tera = setup_tera();
    let db = setup_db_with_migrations("sqlite://app.db?mode=rwc").await;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(db.clone()))
            .service(fs::Files::new("/static", "./templates/static").show_files_listing())
            .configure(init_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
