use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::{ID, Timestamp};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum RoomType {
    #[sea_orm(string_value = "alone")]
    Alone,
    #[sea_orm(string_value = "group")]
    Group,
    #[sea_orm(string_value = "support")]
    Support,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum RoomVisibility {
    #[sea_orm(string_value = "private")]
    Private,
    #[sea_orm(string_value = "public")]
    Public,
    #[sea_orm(string_value = "invite_only")]
    InviteOnly,
}

/// # Room
///
/// The `room` table stores information about rooms.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "room")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    /// # Model Tag
    /// 
    /// The `model_tag` field stores the tag of the model used to generate the message.
    /// This field can change over time, so it is not indexed.
    #[sea_orm(column_type = "Text", column_name = "model_tag", nullable)]
    pub model_tag: Option<String>,

    /// # Template ID
    /// 
    /// Some rooms can be pre builts with prompts, default models, default messages, etc.
    /// This field stores the ID of the template used to generate the message.
    #[sea_orm(column_type = "Uuid", column_name = "template_id", indexed, nullable)]
    pub template_id: Option<ID>,

    /// # Name
    ///
    /// The `name` field stores the name of the room.
    /// It is an optional field, meaning it can be `NULL`.
    #[sea_orm(column_type = "Text", column_name = "name", nullable)]
    pub name: Option<String>,

    #[sea_orm(column_type = "Text", column_name = "description", nullable)]
    pub description: Option<String>,

    #[sea_orm(column_type = "Text", column_name = "icon_url", nullable)]
    pub icon_url: Option<String>,
    #[sea_orm(column_type = "Text", column_name = "background_url", nullable)]
    pub background_url: Option<String>,
    #[sea_orm(column_type = "Text", column_name = "type")]
    pub room_type: RoomType,
    #[sea_orm(column_type = "Text", column_name = "visibility")]
    pub visibility: RoomVisibility,

    #[sea_orm(column_type = "BigInteger", column_name = "deleted_at", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Member,
    Message,
    Template,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Member => Entity::has_many(crate::entities::room::member::Entity)
                .from(Column::Id)
                .to(crate::entities::room::member::Column::RoomId)
                .into(),
            Self::Message => Entity::has_many(crate::entities::room::message::Entity)
                .from(Column::Id)
                .to(crate::entities::room::message::Column::RoomId)
                .into(),
            Self::Template => Entity::has_many(crate::entities::room::template::Entity)
                .from(Column::TemplateId)
                .to(crate::entities::room::template::Column::Id)
                .into(),
        }
    }
}

impl Related<crate::entities::room::member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Member.def()
    }
}

impl Related<crate::entities::room::message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl Related<crate::entities::room::template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
