use actix_files as fs;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};

use migration::{Migrator, MigratorTrait};
use restic_monitor::init_routes;
use sea_orm::Database;
use tera::{Context, Tera};

#[get("/")]
async fn index(tera: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    match tera.render("index.html", &context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let mut tera = Tera::default();
    tera.load_from_glob("templates/**/*.html")
        .expect("Failed to load templates");

    let database_url = "sqlite://app.db?mode=rwc";
    let conn = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");
    Migrator::up(&conn, None).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(conn.clone()))
            .service(index)
            .service(fs::Files::new("/static", "./templates/static").show_files_listing())
            .configure(init_routes)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
