use sea_orm::entity::prelude::*;
use serde::Serialize;

use crate::types::{ID, Timestamp};

/// # Account
///
/// The `account` table stores information about user accounts.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "member")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id",
        indexed
    )]
    pub id: ID,

    #[sea_orm(
        auto_increment = false,
        column_type = "Uuid",
        column_name = "room_id",
        indexed
    )]
    pub room_id: ID,
    #[sea_orm(
        auto_increment = false,
        column_type = "Uuid",
        column_name = "account_id",
        indexed
    )]
    pub account_id: ID,

    pub is_owner: bool,
    pub anonymize: bool,

    #[sea_orm(column_type = "BigInteger", column_name = "banned_at", nullable)]
    pub banned_at: Option<Timestamp>,

    #[sea_orm(column_type = "BigInteger", column_name = "deleted_at", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
    Room,
    Messages,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(crate::entities::account::account::Entity)
                .from(Column::AccountId)
                .to(crate::entities::account::account::Column::Id)
                .into(),
            Self::Room => Entity::belongs_to(crate::entities::room::room::Entity)
                .from(Column::RoomId)
                .to(crate::entities::room::room::Column::Id)
                .into(),
            Self::Messages => Entity::has_many(crate::entities::room::message::Entity)
                .from(Column::Id)
                .to(crate::entities::room::message::Column::MemberId)
                .into(),
        }
    }
}

impl Related<crate::entities::account::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<crate::entities::room::room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Room.def()
    }
}

impl Related<crate::entities::room::message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
