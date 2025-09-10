use std::sync::Arc;

use crate::{
    config,
    entity::{error::AppError, *},
    repository::{mongo::MongoRepository, postgres::PostgresRepository},
};
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
pub trait PostRepository {
    async fn create_post(&self, post: post::Model) -> Result<post::Model, AppError>;
    async fn delete_post_by_id(&self, id: Uuid) -> Result<(), AppError>;
    async fn get_post_by_id(&self, id: Uuid) -> Result<post::Model, AppError>;
    async fn get_posts(&self, query: post::Pagination) -> Result<Vec<post::Model>, AppError>;
    async fn update_post(&self, post: post::Model) -> Result<post::Model, AppError>;
}

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, user: user::Model) -> Result<user::Model, AppError>;
    async fn delete_user_by_id(&self, id: Uuid) -> Result<(), AppError>;
    async fn get_user_by_email(&self, email: String) -> Result<user::Model, AppError>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<user::Model, AppError>;
}

#[async_trait]
pub trait Repository: PostRepository + Send + Sync + UserRepository {
    async fn check_health(&self) -> Result<(), AppError>;
    async fn clear(&self) -> Result<(), AppError>;
    async fn init(&self) -> Result<(), AppError>;
}

pub async fn new() -> Result<Arc<dyn Repository>, AppError> {
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
        other => Err(AppError::Internal {
            err: format!("Invalid database type {other}"),
            path: format!("{}:{}:{}", file!(), line!(), column!()),
        }),
    }
}
