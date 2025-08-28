use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use mongodb::{bson::*, Client};

use crate::config;
use crate::entity::*;
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
    async fn create_user(&self, user: user::Model) -> Result<user::Model> {
        let user = &mut user.clone();
        let user = user.set_id(oid::ObjectId::new().to_hex()).set_created_at(chrono::Utc::now()).clone();
        let result = self.client
        .database(&config::database_name())
        .collection::<user::Model>(config::COLLECTION_USERS)
        .insert_one(&user).await;

        match result {
            Ok(_) => Ok(user),
            Err(err) => {
                if err.to_string().contains("11000") {
                    return Err(anyhow!("User '{}' already exists", &user.email));
                }
                Err(err.into())
            },
        }
    }

    async fn get_user_by_email(&self, email: String) -> Result<user::Model> {
        let result = self.client
        .database(&config::database_name())
        .collection::<user::Model>(config::COLLECTION_USERS)
        .find_one(doc! {"email": &email}).await?;
        match result {
            Some(user) =>  Ok(user),
            None => Err(anyhow!("User '{}' not found", &email)),
        }
    }

    async fn get_user_by_id(&self, id: String) -> Result<user::Model> {
        let result = self.client
        .database(&config::database_name())
        .collection::<user::Model>(config::COLLECTION_USERS)
        .find_one(doc! {"_id": &id}).await?;
        match result {
            Some(user) =>  Ok(user),
            None => Err(anyhow!("User '{}' not found", &id)),
        }
    }
}

impl MongoRepository {
    pub fn new(client: Client) -> Arc<dyn Repository> {
        Arc::new(MongoRepository { client })
    }
}