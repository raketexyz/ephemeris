use actix_cors::Cors;
use actix_web::{http::header, App, HttpServer};
use ephemeris::{db, routes::init_routes};
use log::info;
use std::env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    db::init();

    let server = HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![header::ACCEPT, header::CONTENT_TYPE])
            .max_age(3600);

        App::new().wrap(cors).configure(init_routes)
    });

    let host = env::var("HOST").expect("Host not set");
    let port = env::var("PORT").unwrap_or_else(|_| {
        info!("Port not set, defaulting to 5000.");
        "5000".to_string()
    });
    info!("Starting server on {}:{}", host, port);
    server.bind(format!("{}:{}", host, port))?.run().await
}
