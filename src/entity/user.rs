
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[serde(alias = "_id", rename = "_id")]
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: String,
    #[serde(alias = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(unique)]
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub password: String,
}

impl Model {
    pub fn set_created_at(&mut self, value: chrono::DateTime<chrono::Utc>) -> &mut Self {
        self.created_at = value;
        return self;
    }

    pub fn set_id(&mut self, value: String) -> &mut Self {
        self.id = value;
        return self;
    }

    pub fn set_password(&mut self, value: String) -> &mut Self {
        self.password = value;
        return self;
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