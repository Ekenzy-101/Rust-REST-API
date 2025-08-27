use std::sync::Arc;

use anyhow::{Ok, Result};
use async_trait::async_trait;
use sea_orm::prelude::*;

use crate::repository::*;

#[derive(Clone)]
pub struct PostgresRepository {
    client: DatabaseConnection
}

#[async_trait]
impl Repository for PostgresRepository  {
    async fn check_health(&self) -> Result<()> {
        self.client.ping().await?;
        Ok(())
    }    
}

#[async_trait]
impl UserRepository for PostgresRepository {
    
}

impl PostgresRepository {
    pub fn new(client: DatabaseConnection) -> Arc<dyn Repository> {
        return Arc::new(PostgresRepository { client });
    }
}