use crate::types::{ID, Timestamp};
use sea_orm::entity::prelude::*;
use serde::{self, Deserialize, Serialize};

/// # Account and Flag Relationship
///
/// The
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sea_orm(table_name = "account_has_flag")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "account_id"
    )]
    pub account_id: ID,
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "flag_id"
    )]
    pub flag_id: ID,

    #[sea_orm(column_type = "Boolean", column_name = "system_provided")]
    pub system_provided: bool,

    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
    Flag,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(super::account::Entity)
                .from(Column::AccountId)
                .to(super::account::Column::Id)
                .into(),
            Self::Flag => Entity::belongs_to(super::flag::Entity)
                .from(Column::FlagId)
                .to(super::flag::Column::Id)
                .into(),
        }
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::flag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Flag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
