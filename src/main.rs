pub mod adapter;
pub mod config;
pub mod controller;
pub mod entity;
pub mod repository;

use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};

pub struct AppState {
    repo: Arc<dyn repository::Repository>,
    auth: Arc<adapter::Auth>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let app_data = web::Data::new(AppState {
        repo: repository::new().await.unwrap(),
        auth: adapter::Auth::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(Logger::default())
            .service(controller::check_health)
            .service(controller::create_post)
            .service(controller::delete_post)
            .service(controller::get_post)
            .service(controller::get_posts)
            .service(controller::update_post)
            .service(controller::get_auth_user)
            .service(controller::login_user)
            .service(controller::register_user)
    })
    .workers(1)
    .bind(format!("127.0.0.1:{}", config::port()))?
    .run()
    .await
}
