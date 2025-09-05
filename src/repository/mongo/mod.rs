use std::sync::Arc;

use async_trait::async_trait;
use mongodb::{Client, bson::doc};

use crate::config;
use crate::entity::*;
use crate::repository::*;

#[derive(Clone)]
pub struct MongoRepository {
    client: Client,
}

#[async_trait]
impl Repository for MongoRepository {
    async fn check_health(&self) -> Result<(), AppError> {
        self.client
            .database("admin")
            .run_command(doc! {"ping": 1})
            .await?;
        Ok(())
    }

    async fn init(&self) -> Result<(), AppError> {
        let options = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(options.clone())
            .build();
        self.client
            .database(&config::database_name())
            .collection::<user::Model>(config::COLLECTION_USERS)
            .create_index(index)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl PostRepository for MongoRepository {
    async fn create_post(&self, post: post::Model) -> Result<post::Model, AppError> {
        self.client
            .database(&config::database_name())
            .collection::<post::Model>(config::COLLECTION_POSTS)
            .insert_one(&post)
            .await?;
        Ok(post)
    }

    async fn delete_post_by_id(&self, id: Uuid) -> Result<(), AppError> {
        self.client
            .database(&config::database_name())
            .collection::<post::Model>(config::COLLECTION_POSTS)
            .delete_one(doc! {"_id": id.to_string()})
            .await?;
        Ok(())
    }

    async fn get_post_by_id(&self, id: Uuid) -> Result<post::Model, AppError> {
        let pipeline = vec![
            doc! {"$match": {"_id": id.to_string()}},
            doc! {"$lookup": {
                    "from": config::COLLECTION_USERS,
                    "let": {"user_id": "$user_id"},
                    "pipeline": [
                        {"$match": {"$expr": {"$eq": ["$_id", "$$user_id"]}}},
                        {"$project": {"name": 1, "email": 1, "created_at": 1}},
                    ],
                    "as": "user"
                },
            },
            doc! {"$unwind": "$user"},
            doc! {"$project": {"user_id": 0}},
        ];
        let mut cursor = self
            .client
            .database(&config::database_name())
            .collection::<post::Model>(config::COLLECTION_POSTS)
            .aggregate(pipeline)
            .with_type::<post::Model>()
            .await?;
        while cursor.advance().await? {
            let post = cursor.deserialize_current()?;
            return Ok(post);
        }

        Err(AppError::NotFound(format!("Post '{id}' not found")))
    }

    async fn get_posts(&self, filter: post::Pagination) -> Result<Vec<post::Model>, AppError> {
        let mut pipeline = vec![
            doc! {"$sort": {"_id": -1}},
            doc! {"$skip": filter.offset as i64},
            doc! {"$limit": filter.limit as i64},
            doc! {"$lookup": {
                    "from": config::COLLECTION_USERS,
                    "let": {"user_id": "$user_id"},
                    "pipeline": [
                        {"$match": {"$expr": {"$eq": ["$_id", "$$user_id"]}}},
                        {"$project": {"name": 1, "email": 1, "created_at": 1}},
                    ],
                    "as": "user"
                },
            },
            doc! {"$unwind": "$user"},
            doc! {"$project": {"user_id": 0}},
        ];
        if let Some(user_id) = filter.user_id {
            pipeline.insert(0, doc! {"$match": {"user_id": user_id.to_string()}});
        }

        let mut cursor = self
            .client
            .database(&config::database_name())
            .collection::<post::Model>(config::COLLECTION_POSTS)
            .aggregate(pipeline)
            .with_type::<post::Model>()
            .await?;
        let mut posts = vec![];
        while cursor.advance().await? {
            posts.push(cursor.deserialize_current()?);
        }

        Ok(posts)
    }

    async fn update_post(&self, post: post::Model) -> Result<post::Model, AppError> {
        let update = doc! {
            "$set": {
                "content": &post.content,
                "title": &post.title,
                "updated_at": post.updated_at.to_string(),
            }
        };
        self.client
            .database(&config::database_name())
            .collection::<post::Model>(config::COLLECTION_POSTS)
            .update_one(doc! {"_id": post.id.to_string()}, update)
            .await?;
        Ok(post)
    }
}

#[async_trait]
impl UserRepository for MongoRepository {
    async fn create_user(&self, user: user::Model) -> Result<user::Model, AppError> {
        self.client
            .database(&config::database_name())
            .collection::<user::Model>(config::COLLECTION_USERS)
            .insert_one(&user)
            .await?;
        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> Result<user::Model, AppError> {
        let result = self
            .client
            .database(&config::database_name())
            .collection::<user::Model>(config::COLLECTION_USERS)
            .find_one(doc! {"email": &email})
            .await?;
        match result {
            Some(user) => Ok(user),
            None => Err(AppError::NotFound(format!("User '{email}' not found"))),
        }
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<user::Model, AppError> {
        let result = self
            .client
            .database(&config::database_name())
            .collection::<user::Model>(config::COLLECTION_USERS)
            .find_one(doc! {"_id": id.to_string()})
            .await?;
        match result {
            Some(user) => Ok(user),
            None => Err(AppError::NotFound(format!("User '{id}' not found"))),
        }
    }
}

impl MongoRepository {
    pub fn new(client: Client) -> Arc<dyn Repository> {
        Arc::new(MongoRepository { client })
    }
}
