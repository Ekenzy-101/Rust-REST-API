
use std::sync::Arc;

use anyhow::{Result, Error};
use async_trait::async_trait;
use mongodb::{options::ClientOptions, Client};
use sea_orm::{ConnectOptions, Database};

use crate::{config, repository::{mongo::MongoRepository, postgres::PostgresRepository}};

mod postgres;
mod mongo;

#[async_trait]
pub trait UserRepository {}

pub trait PostRepository {}

#[async_trait]
pub trait Repository: Send + Sync + UserRepository {
    async fn check_health(&self) -> Result<()>;
}

pub async fn new() -> Result<Arc<dyn Repository>> {
    match config::database_type().as_str() {
        "mongo" => {
            let options = ClientOptions::parse(config::database_url()).await?;
            let client = Client::with_options(options)?;
            return Ok(MongoRepository::new(client));
        }
        "postgres" => {
            let options = ConnectOptions::new(config::database_url());
            let client = Database::connect(options).await?;
            return Ok(PostgresRepository::new(client));
        }
        &_ => {
            return Err(Error::msg("invalid database type"));
        }
    }
}