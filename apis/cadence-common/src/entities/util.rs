use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, EntityTrait, Schema};
use tracing::info;

// Import all the Entity types from your entities modules
use crate::entities::{
    account::{account, account_email, account_flag, email, external_identity, flag}, country, room::{member, message, room, template}, tag
};

/// Creates all necessary database tables for the application entities if they don't exist.
///
/// # Arguments
///
/// * `db` - A reference to the active `DatabaseConnection`.
///
/// # Errors
///
/// Returns a `DbErr` if any table creation fails.
pub async fn create_tables_if_not_exists(db: &DatabaseConnection) -> Result<(), DbErr> {
    info!("Setting up database tables...");

    // Get the database backend type
    let db_backend = db.get_database_backend();
    // Get a schema manager
    let schema_manager = Schema::new(db_backend);

    // Helper function to build and execute a create table statement
    async fn create_table<E: EntityTrait>(
        db: &DatabaseConnection,
        schema_manager: &Schema,
        db_backend: DbBackend,
    ) -> Result<(), DbErr> {
        let t = E::default();
        let table_name = E::table_name(&t); // Get table name safely
        info!("Creating table (if not exists): {}", table_name);
        let stmt = schema_manager
            .create_table_from_entity(E::default())
            .if_not_exists()
            .to_owned(); // Add IF NOT EXISTS
        db.execute(db_backend.build(&stmt)).await?; // Build the statement before executing
        Ok(())
    }

    // --- Common Tables ---
    create_table::<tag::Entity>(db, &schema_manager, db_backend).await?;

    // --- Account Related Tables ---
    create_table::<country::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<account::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<email::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<account_email::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<flag::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<account_flag::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<external_identity::Entity>(db, &schema_manager, db_backend).await?;

    // --- Room Related Tables ---
    create_table::<room::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<member::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<template::Entity>(db, &schema_manager, db_backend).await?;
    create_table::<message::Entity>(db, &schema_manager, db_backend).await?;

    info!("Database table setup complete.");
    Ok(())
}
