use actix_web::{HttpResponse, Responder, get, web};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tera::{Context, Tera};

mod entity;
mod error;
mod initial_setup;
mod testing;

#[get("/")]
async fn index(tera: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    match tera.render("index.html", &context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    initial_setup::init_routes(cfg);
}

pub fn setup_tera() -> Tera {
    let mut tera = Tera::default();
    tera.load_from_glob("templates/**/*.html")
        .expect("Failed to load templates");
    return tera;
}

pub async fn setup_db_with_migrations(database_url: &str) -> DatabaseConnection {
    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(2).sqlx_logging(true);
    let conn = Database::connect(opt)
        .await
        .expect("Failed to connect to database");
    Migrator::up(&conn, None).await.unwrap();
    conn
}
