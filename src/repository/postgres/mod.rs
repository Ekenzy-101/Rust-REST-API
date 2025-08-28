use std::sync::Arc;

use anyhow::{anyhow, Ok, Result};
use async_trait::async_trait;
use sea_orm::prelude::*;
use sea_orm::IntoActiveModel;

use crate::entity::*;
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
    async fn create_user(&self, user: user::Model) -> Result<user::Model> {
        let user = &mut user.clone();
        let user = user.set_id(Uuid::new_v4().to_string())
        .set_created_at(chrono::Utc::now())
        .clone().into_active_model()
        .insert(&self.client).await?;
        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> Result<user::Model> {
        let result = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&self.client).await?;
        match result {
            Some(user) =>  Ok(user),
            None => Err(anyhow!("User '{}' not found", &email)),
        }
    }

    async fn get_user_by_id(&self, id: String) -> Result<user::Model> {
        let result = user::Entity::find_by_id(&id)
        .one(&self.client).await?;
        match result {
            Some(user) =>  Ok(user),
            None => Err(anyhow!("User '{}' not found", &id)),
        }
    }
}

impl PostgresRepository {
    pub fn new(client: DatabaseConnection) -> Arc<dyn Repository> {
        Arc::new(PostgresRepository { client })
    }
}