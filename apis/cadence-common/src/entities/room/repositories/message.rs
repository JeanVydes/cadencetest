use crate::entities::room::message::ActiveModel;
use crate::entities::room::message::Column;
use crate::entities::room::message::Entity;
use crate::entities::room::message::MessageType;
use crate::entities::room::message::Model;
use crate::entities::room::message::PrimaryKey;
use crate::repository_traits::CrudEntityRepository;
use crate::time::now_millis;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use serde::Deserialize;
use serde::Serialize;

/// # Message Repository
///
/// This struct provides a repository for managing messages.
#[derive(Clone, Debug)]
pub struct MessageRepository {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreationSchema {
    pub room_id: uuid::Uuid,
    pub member_id: Option<uuid::Uuid>,
    pub system: bool,
    pub model_tag: Option<String>,
    pub content: Option<String>,
    pub attachment: Option<String>,
    pub reply_to: Option<uuid::Uuid>,
    pub message_type: MessageType,
    pub is_hidden: bool,
}

#[async_trait::async_trait]
impl CrudEntityRepository<Model, Entity, ActiveModel, Column, PrimaryKey> for MessageRepository {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type CreationSchema = CreationSchema;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        MessageRepository { db }
    }

    fn db(&self) -> &Self::DatabaseConnection {
        &self.db
    }

    fn deleted_at_column(&self) -> Column {
        Column::DeletedAt
    }

    fn updated_at_column(&self) -> Column {
        Column::UpdatedAt
    }

    fn primary_key_column(&self) -> Column {
        Column::Id
    }

    fn schema_to_active_model(&self, schema: CreationSchema) -> ActiveModel {
        ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            room_id: Set(schema.room_id),
            member_id: Set(schema.member_id),
            system: Set(schema.system),
            model_tag: Set(schema.model_tag),
            content: Set(schema.content),
            attachment: Set(schema.attachment),
            reply_to: Set(schema.reply_to),
            message_type: Set(schema.message_type),
            is_hidden: Set(schema.is_hidden),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        }
    }
}
