pub mod adapter;
pub mod config;
pub mod controller;
pub mod entity;
pub mod repository;
#[cfg(test)]
pub mod tests;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    repo: Arc<dyn repository::Repository>,
    auth: Arc<adapter::Auth>,
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    match config::framework_type().as_str() {
        "actix" => {
            log::debug!("Using Actix as framework");
            controller::actix::main()
        }
        "axum" => {
            log::debug!("Using Axum as framework");
            controller::axum::main()
        }
        _ => {
            log::error!(
                "Please set FRAMEWORK_TYPE environment variable to either 'actix' or 'axum'."
            );
            std::process::exit(1);
        }
    }
}
