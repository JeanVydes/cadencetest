use crate::entities::account::account::ActiveModel;
use crate::entities::account::account::Column;
use crate::entities::account::account::Entity;
use crate::entities::account::account::Model;
use crate::entities::account::account::PrimaryKey;
use crate::repository_traits::CrudEntityRepository;
use crate::types::ID;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use crate::time::now_millis;

/// # Account Repository
///
/// This struct provides a repository for managing accounts.
#[derive(Clone, Debug)]
pub struct AccountRepository {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreationSchema {
    pub name: Option<String>,
    pub country_code_id: ID,
    pub password: String,
}

#[async_trait::async_trait]
impl CrudEntityRepository<Model, Entity, ActiveModel, Column, PrimaryKey> for AccountRepository {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type CreationSchema = CreationSchema;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        AccountRepository { db }
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
            country_code_id: Set(schema.country_code_id),
            password: Set(schema.password),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        }
    }
}
