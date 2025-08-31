use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(alias = "id", rename = "_id", skip_serializing_if = "Uuid::is_nil")]
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[serde(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(unique)]
    pub email: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub password: String,
    #[serde(default)]
    #[sea_orm(ignore)]
    pub posts: Vec<super::post::Model>,
}

impl Model {
    pub fn set_password(mut self, value: String) -> Self {
        self.password = value;
        return self;
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            created_at: chrono::Utc::now(),
            email: String::new(),
            name: String::new(),
            password: String::new(),
            posts: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
