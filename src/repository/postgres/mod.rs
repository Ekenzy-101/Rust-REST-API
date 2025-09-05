use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{IntoActiveModel, Schema, prelude::*};
use sea_orm::{QueryOrder, QuerySelect};

use crate::entity::*;
use crate::repository::*;

#[derive(Clone)]
pub struct PostgresRepository {
    client: DatabaseConnection,
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn check_health(&self) -> Result<(), AppError> {
        self.client.ping().await?;
        Ok(())
    }

    async fn init(&self) -> Result<(), AppError> {
        let mut schema =
            Schema::new(self.client.get_database_backend()).create_table_from_entity(user::Entity);
        self.client
            .execute(
                self.client
                    .get_database_backend()
                    .build(schema.if_not_exists()),
            )
            .await?;

        schema =
            Schema::new(self.client.get_database_backend()).create_table_from_entity(post::Entity);
        self.client
            .execute(
                self.client
                    .get_database_backend()
                    .build(schema.if_not_exists()),
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl PostRepository for PostgresRepository {
    async fn create_post(&self, post: post::Model) -> Result<post::Model, AppError> {
        let post = post.into_active_model().insert(&self.client).await?;
        Ok(post)
    }

    async fn delete_post_by_id(&self, id: Uuid) -> Result<(), AppError> {
        post::Entity::delete_by_id(id).exec(&self.client).await?;
        Ok(())
    }

    async fn get_post_by_id(&self, id: Uuid) -> Result<post::Model, AppError> {
        let result = post::Entity::find_by_id(id)
            .find_also_related(user::Entity)
            .one(&self.client)
            .await?;
        match result {
            Some((post, Some(user))) => Ok(post.set_user(user.set_password("".into())).clone()),
            None => Err(AppError::NotFound(format!("Post '{id}' not found"))),
            _ => Err(AppError::NotFound(format!("Post '{id}' doesn't have user"))),
        }
    }

    async fn get_posts(&self, filter: post::Pagination) -> Result<Vec<post::Model>, AppError> {
        let mut query = post::Entity::find()
            .limit(filter.limit)
            .offset(filter.offset)
            .order_by_desc(post::Column::Id);
        if let Some(user_id) = filter.user_id {
            query = query.filter(post::Column::UserId.eq(user_id));
        }

        let result = query
            .find_also_related(user::Entity)
            .all(&self.client)
            .await?;
        let mut posts = vec![];
        for (mut post, user) in result {
            if user.is_some() {
                post = post.set_user(user.unwrap().set_password("".into()));
                posts.push(post);
            }
        }

        Ok(posts)
    }

    async fn update_post(&self, post: post::Model) -> Result<post::Model, AppError> {
        let post = post
            .into_active_model()
            .reset_all()
            .update(&self.client)
            .await?;
        Ok(post)
    }
}

#[async_trait]
impl UserRepository for PostgresRepository {
    async fn create_user(&self, user: user::Model) -> Result<user::Model, AppError> {
        let user = user.into_active_model().insert(&self.client).await?;
        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> Result<user::Model, AppError> {
        let result = user::Entity::find()
            .filter(user::Column::Email.eq(&email))
            .one(&self.client)
            .await?;
        match result {
            Some(user) => Ok(user),
            None => Err(AppError::NotFound(format!("User '{email}' not found"))),
        }
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<user::Model, AppError> {
        let result = user::Entity::find_by_id(id).one(&self.client).await?;
        match result {
            Some(user) => Ok(user),
            None => Err(AppError::NotFound(format!("User '{id}' not found"))),
        }
    }
}

impl PostgresRepository {
    pub fn new(client: DatabaseConnection) -> Arc<dyn Repository> {
        Arc::new(PostgresRepository { client })
    }
}
