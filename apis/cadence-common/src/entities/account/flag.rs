use crate::types::ID;
use crate::types::Timestamp;
use sea_orm::entity::prelude::*;
use serde::{self, Deserialize, Serialize};

/// # Account Flag
///
/// This entity represents a flag that can be assigned to an account.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sea_orm(table_name = "account_flag")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    #[sea_orm(column_type = "Text", column_name = "name", nullable)]
    pub name: String,

    #[sea_orm(column_type = "BigInteger", column_name = "end_time", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AccountFlag,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AccountFlag => Entity::has_many(crate::entities::account::account_flag::Entity)
                .from(Column::Id)
                .to(crate::entities::account::account_flag::Column::FlagId)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
