
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use mongodb::{bson::doc, options::{ClientOptions, IndexOptions}, Client, IndexModel};
use sea_orm::{ConnectOptions, Database};
use crate::{config, entity::*, repository::{mongo::MongoRepository, postgres::PostgresRepository}};

mod postgres;
mod mongo;

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, user: user::Model) -> Result<user::Model>;
    async fn get_user_by_email(&self, email: String) -> Result<user::Model>;
    async fn get_user_by_id(&self, id: String) -> Result<user::Model>;
}

#[async_trait]
pub trait Repository: Send + Sync + UserRepository {
    async fn check_health(&self) -> Result<()>;
}

pub async fn new() -> Result<Arc<dyn Repository>> {
    match config::database_type().as_str() {
        "mongo" => {
            let options = ClientOptions::parse(config::database_url()).await?;
            let client = Client::with_options(options)?;

            let options = IndexOptions::builder().unique(true).build();
            let index = IndexModel::builder().keys(doc! { "email": 1 }).options(options).build();
            client
            .database(&config::database_name())
            .collection::<user::Model>(config::COLLECTION_USERS)
            .create_index(index).await?;

            Ok(MongoRepository::new(client))
        }
        "postgres" => {
            let options = ConnectOptions::new(config::database_url());
            let client = Database::connect(options).await?;
            Ok(PostgresRepository::new(client))
        }
        &_ => {
            Err(anyhow!("invalid database type"))
        }
    }
}