mod post;
mod user;

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};

pub use post::*;
pub use user::*;

use crate::{AppState, adapter, config, repository};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let state = Arc::new(AppState {
        repo: repository::new().await.unwrap(),
        auth: adapter::Auth::new(),
    });
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", config::port()))
        .await
        .unwrap();
    log::info!("listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, new_app(state)).await
}

pub fn new_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/auth/me", get(get_auth_user))
        .route("/auth/login", post(login_user))
        .route("/auth/register", post(register_user))
        .route("/posts", post(create_post).get(get_posts))
        .route(
            "/posts/{id}",
            delete(delete_post).get(get_post).put(update_post),
        )
        .with_state(state)
}
