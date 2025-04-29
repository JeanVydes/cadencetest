use crate::entities::account::email::ActiveModel;
use crate::entities::account::email::Column;
use crate::entities::account::email::Entity;
use crate::entities::account::email::Model;
use crate::entities::account::email::PrimaryKey;
use crate::error::DatabaseError;
use crate::repository_traits::CrudEntityRepository;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use crate::time::now_millis;

/// # Email Repository
///
/// This struct provides a repository for managing emails.
#[derive(Clone, Debug)]
pub struct EmailRepository {
    pub db: sea_orm::DatabaseConnection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreationSchema {
    pub email: String,
    pub primary: bool,
    pub verification_code: Option<String>,
}

impl EmailRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<Option<Model>, DatabaseError> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(self.db())
            .await
            .map_err(|_| DatabaseError::QueryFailed("Error fetching email".to_owned()))
    }
}

#[async_trait::async_trait]
impl CrudEntityRepository<Model, Entity, ActiveModel, Column, PrimaryKey> for EmailRepository {
    type DatabaseConnection = sea_orm::DatabaseConnection;
    type CreationSchema = CreationSchema;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        EmailRepository { db }
    }

    fn db(&self) -> &DatabaseConnection {
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
            email: Set(schema.email),
            primary: Set(schema.primary),
            verification_code: Set(schema.verification_code),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        }
    }
}
