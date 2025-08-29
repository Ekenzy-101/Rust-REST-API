use std::sync::Arc;

use crate::{
    config,
    entity::*,
    repository::{mongo::MongoRepository, postgres::PostgresRepository},
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use mongodb::{
    Client, IndexModel,
    options::{ClientOptions, IndexOptions},
};
use sea_orm::{ConnectOptions, Database};
use uuid::Uuid;

mod mongo;
mod postgres;

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, user: user::Model) -> Result<user::Model>;
    async fn get_user_by_email(&self, email: String) -> Result<user::Model>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<user::Model>;
}

#[async_trait]
pub trait Repository: Send + Sync + UserRepository {
    async fn check_health(&self) -> Result<()>;
    async fn init(&self) -> Result<()>;
}

pub async fn new() -> Result<Arc<dyn Repository>> {
    match config::database_type().as_str() {
        "mongo" => {
            let options = ClientOptions::parse(config::database_url()).await?;
            let client = Client::with_options(options)?;
            let repo = MongoRepository::new(client);
            repo.init().await?;
            Ok(repo)
        }
        "postgres" => {
            let options = ConnectOptions::new(config::database_url());
            let client = Database::connect(options).await?;
            let repo = PostgresRepository::new(client);
            repo.init().await?;
            Ok(repo)
        }
        &_ => Err(anyhow!("invalid database type")),
    }
}
