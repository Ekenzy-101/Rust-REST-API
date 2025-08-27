pub mod config;
pub mod controller;
pub mod repository;
pub mod entity;

use actix_web::{middleware::Logger, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let repo = repository::new().await.unwrap();
   
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(repo.clone()))
            .wrap(Logger::default())
            .service(controller::check_health)
    })
    .bind(format!("127.0.0.1:{}", config::port()))?
    .run()
    .await
}