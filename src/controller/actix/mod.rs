mod others;
mod post;
mod user;

pub use others::*;
pub use post::*;
pub use user::*;

use actix_web::{App, HttpServer, middleware::Logger, web};

use crate::{AppState, adapter, config, repository};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        repo: repository::new().await.unwrap(),
        auth: adapter::Auth::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(Logger::default())
            .service(check_health)
            .service(create_post)
            .service(delete_post)
            .service(get_post)
            .service(get_posts)
            .service(update_post)
            .service(get_auth_user)
            .service(login_user)
            .service(register_user)
    })
    .workers(1)
    .bind(format!("127.0.0.1:{}", config::port()))?
    .run()
    .await
}
