use crate::auth::AuthService;
use actix_web::{HttpResponse, Responder, get, post, web};

use serde::Deserialize;
use tera::{Context, Tera};

#[allow(dead_code)]
async fn initial_setup_needed(auth_service: &AuthService) -> bool {
    !auth_service.has_admin().await
}

#[derive(Deserialize)]
struct SetupFormData {
    password: String,
}

#[get("/setup")]
async fn initial_setup_page(
    auth_service: web::Data<AuthService>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if !initial_setup_needed(&auth_service).await {
        return HttpResponse::TemporaryRedirect()
            .insert_header((actix_web::http::header::LOCATION, "/"))
            .finish();
    };
    let context = Context::new();
    match tera.render("pages/setup.html", &context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/setup")]
async fn setup_admin_user(
    form: web::Form<SetupFormData>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, crate::error::RustMonitorError> {
    if !initial_setup_needed(&auth_service).await {
        Ok(HttpResponse::SeeOther()
            .insert_header((actix_web::http::header::LOCATION, "/login"))
            .finish())
    } else {
        auth_service.add_user("admin", &form.password).await?;
        Ok(HttpResponse::Created()
            .insert_header((actix_web::http::header::LOCATION, "/login"))
            .finish())
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(initial_setup_page).service(setup_admin_user);
}

#[cfg(test)]
mod test {
    use crate::{auth::AuthService, initial_setup::initial_setup_needed, testing::setup_test_app};

    use scraper::{Html, Selector};

    use serde::Serialize;
    use sqlx::SqlitePool;

    #[derive(Serialize)]
    struct SetupForm<'a> {
        password: &'a str,
    }

    #[sqlx::test]
    async fn without_admin_user_initial_setup_need_detected(pool: SqlitePool) {
        let auth_service = AuthService::new(pool);
        let result = initial_setup_needed(&auth_service).await;
        assert!(result);
    }

    #[sqlx::test]
    async fn with_admin_user_initial_setup_not_need_detected(pool: SqlitePool) {
        let auth_service = AuthService::new(pool);
        let _ = auth_service.add_user("admin", "foobar").await;

        let result = initial_setup_needed(&auth_service).await;
        assert!(!result);
    }

    #[sqlx::test]
    async fn setup_page_renders_correctly_with_form(pool: SqlitePool) {
        let server = setup_test_app(pool).await;
        let response = server.get("/setup").await;

        let html = Html::parse_document(response.text().as_str());
        let form_selector = Selector::parse("form").unwrap();
        let form = html.select(&form_selector).next().unwrap();
        assert_eq!(form.attr("method"), Some("post"));
        assert_eq!(form.attr("action"), Some("/setup"));
        let button = form
            .select(&Selector::parse("button[type=submit]").unwrap())
            .next()
            .unwrap();
        let password_field = form
            .select(&Selector::parse("input[name=password]").unwrap())
            .next();
        assert!(password_field.is_some());
        assert_eq!(button.text().collect::<Vec<&str>>().join(" "), "Save");
    }

    #[sqlx::test]
    async fn setup_submit_form_creates_admin_if_not_exist(pool: SqlitePool) {
        let server = setup_test_app(pool.clone()).await;
        server
            .post("/setup")
            .form(&SetupForm {
                password: "supersecret",
            })
            .expect_success()
            .await;

        let auth_service = AuthService::new(pool);
        let _ = auth_service.verify_user("admin", "supersecret").await;

        assert!(!initial_setup_needed(&auth_service).await);
    }
}
