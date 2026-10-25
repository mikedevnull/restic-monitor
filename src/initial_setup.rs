use crate::entity::{prelude::User, user};
use actix_web::{HttpResponse, Responder, get, post, web};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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

#[get("/setup")]
async fn initial_setup_page(tera: web::Data<Tera>) -> impl Responder {
    let context = Context::new();
    match tera.render("pages/setup.html", &context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/setup")]
async fn setup_admin_user(_tera: web::Data<Tera>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((actix_web::http::header::LOCATION, "/login"))
        .finish()
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(initial_setup_page).service(setup_admin_user);
}

#[cfg(test)]
mod test {
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database};

    use crate::{entity::user, initial_setup::initial_setup_needed};

    #[actix_web::test]
    async fn signal_initial_setup_without_admin_user() {
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
    async fn no_initial_setup_once_admin_present() {
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
}
