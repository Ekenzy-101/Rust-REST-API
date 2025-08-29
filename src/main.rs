pub mod adapter;
pub mod config;
pub mod controller;
pub mod entity;
pub mod repository;

use actix_web::{App, HttpServer, middleware::Logger, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let repo = repository::new().await.unwrap();
    let auth = adapter::Auth::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(repo.clone()))
            .app_data(web::Data::from(auth.clone()))
            .wrap(Logger::default())
            .service(controller::check_health)
            .service(controller::get_auth_user)
            .service(controller::login_user)
            .service(controller::register_user)
    })
    .workers(1)
    .bind(format!("127.0.0.1:{}", config::port()))?
    .run()
    .await
}
