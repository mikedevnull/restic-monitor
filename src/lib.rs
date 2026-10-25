use actix_web::web;

mod entity;
mod initial_setup;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    initial_setup::init_routes(cfg);
}
