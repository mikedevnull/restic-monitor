use actix_web::{HttpResponse, Responder, get, web};
use tera::{Context, Tera};

pub mod auth;
mod db;
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
    tera
}

pub async fn setup_db_with_migrations(database_url: &str) -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect(database_url).await.unwrap();
    db::migrate(pool.clone())
        .await
        .expect("Database migration failed");
    pool
}
