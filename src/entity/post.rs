use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::Validate;

use crate::entity::user;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "id", rename = "_id", skip_serializing_if = "Uuid::is_nil")]
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sea_orm(ignore)]
    pub user: Option<user::Model>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default, skip_serializing_if = "Uuid::is_nil")]
    #[sea_orm(indexed)]
    pub user_id: Uuid,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Model {
    pub fn set_user(mut self, value: user::Model) -> Self {
        self.user = Some(value);
        self.user_id = Uuid::nil();
        return self;
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            created_at: chrono::Utc::now(),
            content: String::new(),
            title: String::new(),
            updated_at: chrono::Utc::now(),
            user: None,
            user_id: Uuid::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Copy, Debug, Deserialize, Validate)]
pub struct Pagination {
    #[validate(range(min = 1, max = 50))]
    pub limit: u64,
    #[validate(range(min = 0))]
    pub offset: u64,
    pub user_id: Option<Uuid>,
}
