use std::fmt::Debug;

use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult,
    IntoActiveModel, ModelTrait, TransactionTrait,
};

use crate::time::now_millis;
use crate::types::ID;
use crate::util::trace_err;

/// # Repository trait for CRUD operations for Repositories
///
/// This trait defines the basic CRUD operations for a repository.
/// It is generic over the model, entity, active model, and column types.
#[async_trait::async_trait]
pub trait CrudEntityRepository<M, E, A, C, Pk>
where
    M: ModelTrait<Entity = E> + IntoActiveModel<A> + Send + Sync + FromQueryResult, // Model requirements
    E: EntityTrait<Model = M, ActiveModel = A, Column = C, PrimaryKey = Pk> + Send + Sync, // Entity requirements
    A: ActiveModelTrait<Entity = E>
        + ActiveModelBehavior
        + Send
        + Sync
        + Default
        + From<M>
        + 'static, // ActiveModel requirements
    C: ColumnTrait + Send + Sync,
    Pk: PrimaryKeyTrait + Send + Sync,
    <Pk as PrimaryKeyTrait>::ValueType: Eq
        + std::hash::Hash
        + Clone
        + Send
        + Sync
        + sea_orm::sea_query::ValueType
        + From<ID>
        + Into<ID>
        + Into<sea_orm::Value>,
{
    type DatabaseConnection: ConnectionTrait + Send + Sync;
    type CreationSchema: Send + Sync + Clone;

    fn db(&self) -> &Self::DatabaseConnection;
    fn new(db: Self::DatabaseConnection) -> Self;
    fn schema_to_active_model(&self, schema: Self::CreationSchema) -> A;
    fn deleted_at_column(&self) -> C;
    fn updated_at_column(&self) -> C;
    fn primary_key_column(&self) -> C;

    /// Creates a new entity in the database.
    async fn create(&self, schema: &Self::CreationSchema) -> Result<M, DbErr> {
        self.schema_to_active_model(schema.clone())
            .insert(self.db())
            .await
            .map_err(trace_err("Error creating entity"))
    }

    /// Creates a new entity in the database within a transaction.
    async fn create_tx(
        &self,
        schema: &Self::CreationSchema,
        txn: &(impl TransactionTrait + ConnectionTrait),
    ) -> Result<M, DbErr> {
        self.schema_to_active_model(schema.clone())
            .insert(txn)
            .await
            .map_err(trace_err("Error creating entity with transaction"))
    }

    /// Get a single one record by id
    async fn get_by_id(&self, id: ID) -> Result<Option<M>, DbErr> {
        // Use the Pk's ValueType conversion from ID
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();
        Ok(
            E::find_by_id(pk_value) // Use Entity::find_by_id with the correct PK type
                .one(self.db())
                .await
                .map_err(trace_err("Error fetching entity"))?
                .map(|model| {
                    tracing::trace!("Found entity: {:?}", model);
                    model
                }),
        )
    }

    /// Get multuple records by ids
    async fn get_by_ids(&self, ids: Vec<ID>) -> Result<Vec<M>, DbErr> {
        // Convert Vec<ID> to Vec<Pk::ValueType>
        let pk_values: Vec<<Pk as PrimaryKeyTrait>::ValueType> =
            ids.into_iter().map(|id| id.into()).collect();
        let entities = E::find()
            .filter(self.primary_key_column().is_in(pk_values)) // Use primary_key_column
            .all(self.db())
            .await
            .map_err(trace_err("Error fetching entities"))?;

        tracing::trace!("Found entities: {:?}", entities);
        Ok(entities)
    }

    /// Delete a single record by id
    async fn delete(&self, id: ID) -> Result<M, DbErr> {
        // Fetch using the correct PK type
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();
        let model = E::find_by_id(pk_value).one(self.db()).await?;

        if let Some(model) = model {
            let mut active_model: A = model.into();
            let deleted_at_col = self.deleted_at_column();
            active_model.set(deleted_at_col, Value::BigInt(Some(now_millis())));

            active_model
                .update(self.db())
                .await
                .map_err(trace_err("Error soft deleting entity"))
                .map(|model| {
                    tracing::trace!("Soft deleted entity: {:?}", model);
                    model
                })
        } else {
            Err(DbErr::RecordNotFound(format!(
                "Entity with id {:?} not found for deletion", // Use {:?} for potentially complex IDs
                id
            )))
        }
    }

    async fn delete_tx(
        &self,
        id: ID,
        txn: &(impl TransactionTrait + ConnectionTrait),
    ) -> Result<M, DbErr> {
        // Return M
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();
        let model = E::find_by_id(pk_value).one(txn).await.map_err(trace_err("Error fetching entity"))?;

        if let Some(model) = model {
            let mut active_model: A = model.into();
            let deleted_at_col = self.deleted_at_column();
            // Use Set for consistency
            active_model.set(deleted_at_col, Value::BigInt(Some(now_millis())));

            Ok(active_model
                .update(txn)
                .await
                .map_err(trace_err("Error soft deleting entity with transaction"))?)
        } else {
            Err(DbErr::RecordNotFound(format!(
                "Entity with id {:?} not found for deletion in transaction",
                id
            )))
        }
    }

    async fn update(&self, id: ID, mut model: A) -> Result<M, DbErr> {
        let pk_col = self.primary_key_column();
        let updated_at_col = self.updated_at_column();
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();

        model.set(pk_col, pk_value.clone().into());
        model.set(updated_at_col, Value::BigInt(Some(now_millis())));
        Ok(E::update(model).exec(self.db()).await.map_err(trace_err("Error updating entity"))?)
    }

    async fn update_tx(
        &self,
        id: ID,
        mut model: A,
        txn: &(impl TransactionTrait + ConnectionTrait),
    ) -> Result<M, DbErr> {
        let pk_col = self.primary_key_column();
        let updated_at_col = self.updated_at_column();
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();

        model.set(pk_col, pk_value.clone().into());
        model.set(updated_at_col, Value::BigInt(Some(now_millis())));
        Ok(E::update(model).exec(txn).await.map_err(trace_err("Error updating entity"))?)
    }

    async fn exists(&self, id: ID) -> Result<(bool, Option<M>), DbErr> {
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();
        let result = E::find_by_id(pk_value).one(self.db()).await.map_err(trace_err("Error checking existence"))?;

        let deleted_at_col = self.deleted_at_column();
        let exists = result
            .as_ref()
            .map(|model| match model.get(deleted_at_col) {
                Value::BigInt(Some(0)) | Value::BigInt(None) => true,
                _ => false,
            })
            .unwrap_or(false);

        Ok((exists, result))
    }

    async fn exists_tx(
        &self,
        id: ID,
        txn: &(impl TransactionTrait + ConnectionTrait),
    ) -> Result<(bool, Option<M>), DbErr> {
        let pk_value: <Pk as PrimaryKeyTrait>::ValueType = id.into();
        let result = E::find_by_id(pk_value).one(txn).await.map_err(trace_err("Error checking existence"))?;

        let deleted_at_col = self.deleted_at_column();
        let exists = result
            .as_ref()
            .map(|model| match model.get(deleted_at_col) {
                Value::BigInt(Some(0)) | Value::BigInt(None) => true,
                _ => false,
            })
            .unwrap_or(false);

        Ok((exists, result))
    }
}

#[async_trait::async_trait]
pub trait BasicApplicationService
where
    Self: Clone + Send + Sync + Debug,
{
    type DatabaseConnection: ConnectionTrait + Send + Sync;

    fn db(&self) -> &Self::DatabaseConnection;
    fn new(db: Self::DatabaseConnection) -> Self;
}
