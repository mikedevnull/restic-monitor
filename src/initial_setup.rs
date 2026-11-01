use crate::entity::{prelude::User, user};
use actix_web::{HttpResponse, Responder, get, post, web};
use argon2::{Argon2, PasswordHasher};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Deserialize;
use tera::{Context, Tera};

#[allow(dead_code)]
async fn initial_setup_needed(db: &DatabaseConnection) -> bool {
    let user = User::find()
        .filter(user::Column::Name.eq("admin"))
        .one(db)
        .await
        .unwrap_or(None);
    user.is_none()
}

#[derive(Deserialize)]
struct SetupFormData {
    password: String,
}

#[get("/setup")]
async fn initial_setup_page(
    db: web::Data<DatabaseConnection>,
    tera: web::Data<Tera>,
) -> impl Responder {
    if !initial_setup_needed(&db).await {
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
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, crate::error::RustMonitorError> {
    if !initial_setup_needed(&db).await {
        Ok(HttpResponse::SeeOther()
            .insert_header((actix_web::http::header::LOCATION, "/login"))
            .finish())
    } else {
        let hashed_password = Argon2::default()
            .hash_password(form.password.as_bytes())
            .unwrap();
        let user = user::ActiveModel {
            name: Set("admin".to_owned()),
            password: Set(hashed_password.to_string()),
            ..Default::default() // all other attributes are `NotSet`
        };
        let _result = user.insert(db.get_ref()).await?;
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
    use crate::{
        entity::{prelude::User, user},
        initial_setup::initial_setup_needed,
        testing::setup_test_app,
    };

    use argon2::{Argon2, PasswordVerifier};
    use scraper::{Html, Selector};
    use sea_orm::{
        ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, EntityTrait, QueryFilter,
    };
    use serde::Serialize;

    #[derive(Serialize)]
    struct SetupForm<'a> {
        password: &'a str,
    }

    #[actix_web::test]
    async fn without_admin_user_initial_setup_need_detected() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.get_schema_builder()
            .register(user::Entity)
            .apply(&db)
            .await
            .expect("Failed to setup db schema for testing");

        let result = initial_setup_needed(&db).await;
        assert!(result);
    }

    #[actix_web::test]
    async fn with_admin_user_initial_setup_not_need_detected() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        db.get_schema_builder()
            .register(user::Entity)
            .apply(&db)
            .await
            .expect("Failed to setup db schema for testing");
        let admin = user::ActiveModel {
            name: Set("admin".to_owned()),
            password: Set("hashedfoobar".to_owned()),
        };
        admin
            .insert(&db)
            .await
            .expect("Failed to setup fake admin user");

        let result = initial_setup_needed(&db).await;
        assert!(!result);
    }

    #[actix_web::test]
    async fn setup_page_renders_correctly_with_form() {
        let (server, _) = setup_test_app().await;
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

    #[actix_web::test]
    async fn setup_submit_form_creates_admin_if_not_exist() {
        let (server, db) = setup_test_app().await;
        server
            .post("/setup")
            .form(&SetupForm {
                password: "supersecret",
            })
            .expect_success()
            .await;

        let admin = User::find()
            .filter(user::Column::Name.eq("admin"))
            .one(&db)
            .await
            .unwrap()
            .expect("");

        assert!(
            Argon2::default()
                .verify_password("supersecret".as_bytes(), admin.password.as_str())
                .is_ok()
        );
        assert!(!initial_setup_needed(&db).await);
    }
}
