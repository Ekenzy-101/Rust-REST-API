mod others;
mod post;
mod user;

pub use others::*;
pub use post::*;
pub use user::*;

use actix_web::{
    App, HttpServer,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware::Logger,
    web::{self, Data},
};

use crate::{AppState, adapter, config, repository};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        repo: repository::new().await.unwrap(),
        auth: adapter::Auth::new(),
    });

    HttpServer::new(move || new_app(state.clone()).wrap(Logger::default()))
        .bind(config::address())?
        .run()
        .await
}

pub fn new_app(
    state: Data<AppState>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(state)
        .service(check_health)
        .service(create_post)
        .service(delete_post)
        .service(get_post)
        .service(get_posts)
        .service(update_post)
        .service(get_auth_user)
        .service(login_user)
        .service(register_user)
}
