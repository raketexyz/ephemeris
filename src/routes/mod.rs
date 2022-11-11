use actix_web::web;

mod posts;
mod users;
mod comments;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    users::init_routes(cfg);
    posts::init_routes(cfg);
    comments::init_routes(cfg);
}
