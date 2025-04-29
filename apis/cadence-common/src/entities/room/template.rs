use sea_orm::entity::prelude::*;
use serde::Serialize;

use crate::types::{ID, Timestamp};

/// # Template
///
/// The `template` table stores information about templates used in rooms.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "room_template")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    #[sea_orm(column_type = "Uuid", column_name = "author_id", indexed, nullable)]
    pub author_id: Option<ID>,

    #[sea_orm(column_type = "Text", column_name = "system_prompt", nullable)]
    pub system_prompt: Option<String>,

    /// # Model Tag
    /// 
    /// The `model_tag` field stores the tag of the model used to generate the message.
    #[sea_orm(column_type = "Text", column_name = "model_tag", nullable)]
    pub model_tag: String,

    /// # Template Room ID
    /// 
    /// Some rooms can be pre-built with prompts, default models, default messages, etc.
    /// New room can be created from this chat "copying" the room.
    #[sea_orm(
        auto_increment = false,
        column_type = "Uuid",
        column_name = "source_room_id",
        indexed,
        nullable
    )]
    pub source_room_id: Option<ID>,

    /// # Name
    ///
    /// The `name` field stores the name of the template.
    /// It is an optional field, meaning it can be `NULL`.
    #[sea_orm(column_type = "Text", column_name = "name")]
    pub name: Option<String>,
    #[sea_orm(column_type = "Text", column_name = "description", nullable)]
    pub description: Option<String>,

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
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => {
                Entity::belongs_to(crate::entities::account::account::Entity)
                    .from(Column::AuthorId)
                    .to(crate::entities::account::account::Column::Id)
                    .into()
            }
            Self::Room => {
                Entity::belongs_to(crate::entities::room::room::Entity)
                    .from(Column::SourceRoomId)
                    .to(crate::entities::room::room::Column::Id)
                    .into()
            }
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

impl ActiveModelBehavior for ActiveModel {}
