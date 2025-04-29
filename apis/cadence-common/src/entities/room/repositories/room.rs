use crate::entities::room::room::ActiveModel;
use crate::entities::room::room::Column;
use crate::entities::room::room::Entity;
use crate::entities::room::room::Model;
use crate::entities::room::room::PrimaryKey;
use crate::entities::room::room::RoomType;
use crate::entities::room::room::RoomVisibility;
use crate::repository_traits::CrudEntityRepository;
use crate::time::now_millis;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use serde::Deserialize;
use serde::Serialize;

/// # Room Repository
///
/// This struct provides a repository for managing rooms.
#[derive(Clone, Debug)]
pub struct RoomRepository {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreationSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub background_url: Option<String>,
    pub visibility: RoomVisibility,
    pub template_id: Option<uuid::Uuid>,
    pub model_tag: Option<String>,
    pub room_type: RoomType,
}

#[async_trait::async_trait]
impl CrudEntityRepository<Model, Entity, ActiveModel, Column, PrimaryKey> for RoomRepository {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type CreationSchema = CreationSchema;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        RoomRepository { db }
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
            icon_url: Set(schema.icon_url),
            background_url: Set(schema.background_url),
            room_type: Set(schema.room_type),
            visibility: Set(schema.visibility),
            template_id: Set(schema.template_id),
            model_tag: Set(schema.model_tag),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        }
    }
}
