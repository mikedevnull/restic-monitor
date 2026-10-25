use actix_files as fs;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
use env_logger::Env;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let mut tera = Tera::default();
    tera.load_from_glob("templates/**/*.html")
        .expect("Failed to load templates");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(tera.clone()))
            .service(index)
            .service(fs::Files::new("/static", "./templates/static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
