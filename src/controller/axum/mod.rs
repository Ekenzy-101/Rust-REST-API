mod others;
mod post;
mod user;

use std::{net::SocketAddr, sync::Arc, time::Instant};

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
};
use http_body_util::BodyExt;

pub use others::*;
pub use post::*;
pub use user::*;

use crate::{AppState, adapter, config, repository};

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let state = Arc::new(AppState {
        repo: repository::new().await.unwrap(),
        auth: adapter::Auth::new(),
    });
    let listener = tokio::net::TcpListener::bind(config::address())
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
        .route("/health", get(check_health))
        .route("/posts", post(create_post).get(get_posts))
        .route(
            "/posts/{id}",
            delete(delete_post).get(get_post).put(update_post),
        )
        .fallback(not_found)
        .layer(middleware::from_fn(logging_middleware))
        .with_state(state)
}

async fn logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let referer = req
        .headers()
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();
    let start = Instant::now();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();
    let version = req.version();

    let response = next.run(req).await;
    let duration = start.elapsed();
    let status = response.status().as_u16();
    let (parts, body) = response.into_parts();
    let remote_ip = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.to_string())
        .unwrap_or_else(|| "-".into());
    let body_bytes = body.collect().await.unwrap().to_bytes();
    let size = parts
        .headers
        .iter()
        .map(|(k, v)| k.as_str().len() + v.len())
        .sum::<usize>()
        + body_bytes.len();

    log::info!(
        "{} {}",
        format!("{} \"{} {} {:?}\"", remote_ip, method, path, version),
        format!(
            "{} {} \"{}\" \"{}\" {}ms",
            status,
            size,
            referer,
            user_agent,
            duration.as_millis()
        )
    );

    Response::from_parts(parts, Body::from(body_bytes))
}
