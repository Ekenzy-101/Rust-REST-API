use std::sync::Arc;

use anyhow::{Result};
use async_trait::async_trait;
use mongodb::{bson::doc, Client};

use crate::repository::*;

#[derive(Clone)]
pub struct MongoRepository {
    client: Client
}

#[async_trait]
impl Repository for MongoRepository  {
    async fn check_health(&self) -> Result<()> {
        self.client.database("admin").run_command(doc!{"ping": 1}).await?;
        Ok(())
    }
}


#[async_trait]
impl UserRepository for MongoRepository {

}

impl MongoRepository {
    pub fn new(client: Client) -> Arc<dyn Repository> {
        return Arc::new(MongoRepository { client });
    }
}