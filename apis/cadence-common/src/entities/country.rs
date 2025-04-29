use crate::types::ID;
use crate::types::Timestamp;
use sea_orm::entity::prelude::*;
use serde::{self, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sea_orm(table_name = "country")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    #[sea_orm(column_type = "Text", column_name = "name")]
    pub name: String,
    #[sea_orm(column_type = "Text", column_name = "alpha_2")]
    pub alpha_2: String,

    #[sea_orm(column_type = "BigInteger", column_name = "end_time", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::has_many(super::account::account::Entity)
                .from(Column::Id)
                .to(super::account::account::Column::CountryCodeId)
                .into(),
        }
    }
}

impl Related<crate::entities::account::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
