use crate::entities::room::member::ActiveModel;
use crate::entities::room::member::Column;
use crate::entities::room::member::Entity;
use crate::entities::room::member::Model;
use crate::entities::room::member::PrimaryKey;
use crate::repository_traits::CrudEntityRepository;
use crate::time::now_millis;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use serde::Deserialize;
use serde::Serialize;

/// # Member Repository
///
/// This struct provides a repository for managing room members.
#[derive(Clone, Debug)]
pub struct MemberRepository {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreationSchema {
    pub room_id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub is_owner: bool,
    pub anonymize: bool,
}

#[async_trait::async_trait]
impl CrudEntityRepository<Model, Entity, ActiveModel, Column, PrimaryKey> for MemberRepository {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type CreationSchema = CreationSchema;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        MemberRepository { db }
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
            account_id: Set(schema.account_id),
            is_owner: Set(schema.is_owner),
            anonymize: Set(schema.anonymize),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        }
    }
}
