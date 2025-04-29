use crate::entities::room::template::ActiveModel;
use crate::entities::room::template::Column;
use crate::entities::room::template::Entity;
use crate::entities::room::template::Model;
use crate::entities::room::template::PrimaryKey;
use crate::repository_traits::CrudEntityRepository;
use crate::time::now_millis;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use serde::Deserialize;
use serde::Serialize;

/// # Room Template Repository
///
/// This struct provides a repository for managing rooms template.
#[derive(Clone, Debug)]
pub struct RoomTemplateRepository {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreationSchema {
    pub author_id: Option<uuid::Uuid>,
    pub model_tag: String,
    pub source_room_id: Option<uuid::Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
}

#[async_trait::async_trait]
impl CrudEntityRepository<Model, Entity, ActiveModel, Column, PrimaryKey> for RoomTemplateRepository {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type CreationSchema = CreationSchema;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        RoomTemplateRepository { db }
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
            name: Set(schema.name),
            description: Set(schema.description),
            source_room_id: Set(schema.source_room_id),
            author_id: Set(schema.author_id),
            model_tag: Set(schema.model_tag),
            system_prompt: Set(schema.system_prompt),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        }
    }
}
