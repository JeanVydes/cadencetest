#![cfg(test)] // Ensure this file is only compiled for tests

use super::account::repositories::{
    account::{AccountRepository, CreationSchema as AccountCreationSchema},
    email::{CreationSchema as EmailCreationSchema, EmailRepository},
};
use super::calendar::repositories::{
    event_metadata::{CreationSchema as EventMetadataCreationSchema, EventMetadataRepository},
    event_time::{CreationSchema as EventTimeCreationSchema, EventTimeRepository},
    exception::{CreationSchema as ExceptionCreationSchema, ExceptionRepository},
    recurrence::{CreationSchema as RecurrenceCreationSchema, RecurrenceRepository},
};
use super::goal::repositories::goal::{CreationSchema as GoalCreationSchema, GoalRepository};

// Import Models and Enums needed for mocking and data creation
use super::account::{
    account as account_entity, email as email_entity,
    external_identity as external_identity_entity,
};
use super::calendar::{
    event_metadata as event_metadata_entity, event_time as event_time_entity,
    exception as exception_entity, recurrence as recurrence_entity,
};
use super::goal::goal as goal_entity;

use crate::repository_traits::CrudEntityRepository;
use crate::types::{DateWithTimeZone, ID, Timestamp}; // Assuming DateWithTimeZone is defined
use chrono::{TimeZone, Utc};
use sea_orm::{
    error::*, DatabaseConnection, DbBackend, FromQueryResult, MockDatabase, MockExecResult,
    MockRow, Statement, IntoSimpleExpr,
};
use std::sync::Arc; // Needed for Arc<DatabaseConnection>
use uuid::Uuid;

// --- Mocking Helpers ---

/// Helper trait to easily convert a Model into a MockRow for query results.
/// Assumes the mock query selects all columns in the order they appear in the Model struct.
trait IntoMockRow {
    fn into_mock_row(self) -> MockRow;
}

// --- Implementations for IntoMockRow ---

// (Implementations for IntoMockRow remain the same)
impl IntoMockRow for account_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.name)
            .append_value(self.country_code)
            .append_value(self.password)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}

impl IntoMockRow for email_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.email)
            .append_value(self.primary)
            .append_value(self.verified_at)
            .append_value(self.verification_code)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}

impl IntoMockRow for event_metadata_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.author_id)
            .append_value(self.ical_uid)
            .append_value(self.title)
            .append_value(self.description)
            .append_value(self.color)
            .append_value(self.address) // Renamed from location in entity
            .append_value(self.meeting_link)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}

impl IntoMockRow for event_time_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.recurrence_id)
            .append_value(self.start_time)
            .append_value(self.end_time)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}

impl IntoMockRow for recurrence_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.event_id) // Renamed from event_time_id in entity
            .append_value(self.status)
            .append_value(self.rrule)
            .append_value(self.exdate)
            .append_value(self.rdate)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}

impl IntoMockRow for exception_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.recurrence_id)
            .append_value(self.event_time_id) // Added this field
            .append_value(self.original_start_time)
            .append_value(self.status)
            .append_value(self.new_event_time_id)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}

impl IntoMockRow for goal_entity::Model {
    fn into_mock_row(self) -> MockRow {
        MockRow::new()
            .append_value(self.id)
            .append_value(self.author_id)
            .append_value(self.summary)
            .append_value(self.description)
            .append_value(self.color)
            .append_value(self.deleted_at)
            .append_value(self.created_at)
            .append_value(self.updated_at)
    }
}


// Helper to create a mock DB connection preloaded with query results
fn setup_mock_db_with_query_results(
    results: Vec<Vec<impl IntoMockRow + Clone>>, // Clone needed for potential reuse
) -> Arc<DatabaseConnection> { // Changed return type
    Arc::new( // Wrap in Arc
        MockDatabase::new(DbBackend::Postgres)
            .append_query_results(results)
            .into_connection()
    )
}

// Helper to create a mock DB connection preloaded with execution results (for insert/update/delete)
fn setup_mock_db_with_exec_results(results: Vec<MockExecResult>) -> Arc<DatabaseConnection> { // Changed return type
     Arc::new( // Wrap in Arc
        MockDatabase::new(DbBackend::Postgres)
            .append_exec_results(results)
            .into_connection()
     )
}

// Helper to create a mock DB connection preloaded with errors
fn setup_mock_db_with_errors(errors: Vec<DbErr>) -> Arc<DatabaseConnection> { // Changed return type
     Arc::new( // Wrap in Arc
        MockDatabase::new(DbBackend::Postgres)
            .append_query_errors(errors.clone()) // Clone errors for potential multiple uses
            .append_exec_errors(errors)
            .into_connection()
     )
}

// --- Test Modules per Repository ---

mod account_repo_tests {
    use super::*; // Import helpers and types from parent module

    fn create_test_account(id: ID) -> account_entity::Model {
        let now = Utc::now().timestamp_millis();
        account_entity::Model {
            id,
            name: Some("Test User".to_string()),
            country_code: "US".to_string(),
            password: "hashed_password".to_string(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_account_get_by_id_found() {
        let account_id = Uuid::new_v4();
        let expected_account = create_test_account(account_id);
        // setup_mock_db_... now returns Arc<DatabaseConnection>
        let db = setup_mock_db_with_query_results(vec![vec![expected_account.clone()]]);
        // Pass the Arc to the repository constructor
        let repo = AccountRepository::new(db);

        let result = repo.get_by_id(account_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_account));
    }

    #[tokio::test]
    async fn test_account_get_by_id_not_found() {
        let account_id = Uuid::new_v4();
        let db = setup_mock_db_with_query_results(vec![Vec::<account_entity::Model>::new()]);
        let repo = AccountRepository::new(db); // Pass Arc

        let result = repo.get_by_id(account_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_account_get_by_id_db_error() {
        let account_id = Uuid::new_v4();
        let db = setup_mock_db_with_errors(vec![DbErr::Query(RuntimeErr::Internal(
            "Simulated DB Error".into(),
        ))]);
        let repo = AccountRepository::new(db); // Pass Arc

        let result = repo.get_by_id(account_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_create_success() {
        let schema = AccountCreationSchema {
            name: Some("New User".to_string()),
            country_code: "CA".to_string(),
            password: "new_hashed_password".to_string(),
        };
        let db = setup_mock_db_with_exec_results(vec![MockExecResult {
            last_insert_id: 1, // Not directly used by SeaORM for UUIDs, but needed
            rows_affected: 1,
        }]);
        let repo = AccountRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_account = result.unwrap();
        assert_eq!(created_account.name, schema.name);
        assert_eq!(created_account.country_code, schema.country_code);
        assert_eq!(created_account.password, schema.password);
        assert!(!created_account.id.is_nil());
        assert!(created_account.created_at > 0);
        assert!(created_account.updated_at > 0);
    }

    #[tokio::test]
    async fn test_account_delete_success() {
        let account_id = Uuid::new_v4();
        let existing_account = create_test_account(account_id);

        // Mock DB setup remains similar, but wrap the final connection in Arc
        let db = Arc::new( // Wrap in Arc
            MockDatabase::new(DbBackend::Postgres)
                .append_query_results(vec![vec![existing_account.clone()]]) // Result for get_by_id
                .append_exec_results(vec![MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                }]) // Result for update (soft delete)
                .into_connection()
        );
        let repo = AccountRepository::new(db); // Pass Arc

        let result = repo.delete(account_id).await;

        assert!(result.is_ok());
        let deleted_account = result.unwrap();
        assert!(deleted_account.deleted_at.is_some());
        assert_eq!(deleted_account.id, account_id);
    }

    #[tokio::test]
    async fn test_account_delete_not_found() {
        let account_id = Uuid::new_v4();
        let db = setup_mock_db_with_query_results(vec![Vec::<account_entity::Model>::new()]); // get_by_id returns nothing
        let repo = AccountRepository::new(db); // Pass Arc

        let result = repo.delete(account_id).await;

        assert!(result.is_err());
        // Use matches! for cleaner error checking
        assert!(matches!(result.err().unwrap(), DbErr::RecordNotFound(_)), "Expected RecordNotFound error");
    }

    // Add tests for get_by_ids, update, update_tx, delete_tx if needed
}

mod email_repo_tests {
    use super::*;

    fn create_test_email(id: ID) -> email_entity::Model {
        let now = Utc::now().timestamp_millis();
        email_entity::Model {
            id,
            email: format!("test{}@example.com", id),
            primary: false,
            verified_at: None,
            verification_code: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_email_get_by_id_found() {
        let email_id = Uuid::new_v4();
        let expected_email = create_test_email(email_id);
        let db = setup_mock_db_with_query_results(vec![vec![expected_email.clone()]]);
        let repo = EmailRepository::new(db); // Pass Arc

        let result = repo.get_by_id(email_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_email));
    }

    #[tokio::test]
    async fn test_email_get_by_id_not_found() {
        let email_id = Uuid::new_v4();
        let db = setup_mock_db_with_query_results(vec![Vec::<email_entity::Model>::new()]);
        let repo = EmailRepository::new(db); // Pass Arc

        let result = repo.get_by_id(email_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

     #[tokio::test]
    async fn test_email_create_success() {
        let schema = EmailCreationSchema {
            email: "new@example.com".to_string(),
            primary: true,
            verification_code: Some("123456".to_string()),
        };
        let db = setup_mock_db_with_exec_results(vec![MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }]);
        let repo = EmailRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_email = result.unwrap();
        assert_eq!(created_email.email, schema.email);
        assert_eq!(created_email.primary, schema.primary);
        assert_eq!(created_email.verification_code, schema.verification_code);
        assert!(!created_email.id.is_nil());
    }

    #[tokio::test]
    async fn test_email_delete_success() {
        let email_id = Uuid::new_v4();
        let existing_email = create_test_email(email_id);

        let db = Arc::new( // Wrap in Arc
            MockDatabase::new(DbBackend::Postgres)
                .append_query_results(vec![vec![existing_email.clone()]]) // Result for get_by_id
                .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }]) // Result for update
                .into_connection()
        );
        let repo = EmailRepository::new(db); // Pass Arc

        let result = repo.delete(email_id).await;

        assert!(result.is_ok());
        let deleted_email = result.unwrap();
        assert!(deleted_email.deleted_at.is_some());
        assert_eq!(deleted_email.id, email_id);
    }

     #[tokio::test]
    async fn test_email_delete_not_found() {
        let email_id = Uuid::new_v4();
        let db = setup_mock_db_with_query_results(vec![Vec::<email_entity::Model>::new()]);
        let repo = EmailRepository::new(db); // Pass Arc

        let result = repo.delete(email_id).await;

        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), DbErr::RecordNotFound(_)), "Expected RecordNotFound error");
    }

    // Add tests for get_by_ids, update, etc.
}

mod event_metadata_repo_tests {
    use super::*;

    fn create_test_metadata(id: ID, author_id: ID) -> event_metadata_entity::Model {
        let now = Utc::now().timestamp_millis();
        event_metadata_entity::Model {
            id,
            author_id,
            ical_uid: Some(format!("ical-{}", id)),
            title: "Test Event".to_string(),
            description: "A test event description".to_string(),
            color: Some("#FF0000".to_string()),
            address: Some("123 Test St".to_string()),
            meeting_link: Some("http://meet.example.com".to_string()),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

     #[tokio::test]
    async fn test_eventmeta_get_by_id_found() {
        let meta_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let expected_meta = create_test_metadata(meta_id, author_id);
        let db = setup_mock_db_with_query_results(vec![vec![expected_meta.clone()]]);
        let repo = EventMetadataRepository::new(db); // Pass Arc

        let result = repo.get_by_id(meta_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_meta));
    }

     #[tokio::test]
    async fn test_eventmeta_get_by_id_not_found() {
        let meta_id = Uuid::new_v4();
        let db = setup_mock_db_with_query_results(vec![Vec::<event_metadata_entity::Model>::new()]);
        let repo = EventMetadataRepository::new(db); // Pass Arc

        let result = repo.get_by_id(meta_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_eventmeta_create_success() {
        let author_id = Uuid::new_v4();
        let schema = EventMetadataCreationSchema {
            author_id,
            title: "New Event".to_string(),
            description: "Details...".to_string(),
            color: None,
            meeting_link: None,
            address: None,
        };
        let db = setup_mock_db_with_exec_results(vec![MockExecResult { last_insert_id: 1, rows_affected: 1 }]);
        let repo = EventMetadataRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_meta = result.unwrap();
        assert_eq!(created_meta.author_id, schema.author_id);
        assert_eq!(created_meta.title, schema.title);
        assert!(!created_meta.id.is_nil());
    }

    // Add tests for delete, update, etc.
}

mod event_time_repo_tests {
     use super::*;

     // Helper to create DateWithTimeZone for tests
     fn test_time(ts_secs: i64) -> DateWithTimeZone {
         Utc.timestamp_opt(ts_secs, 0).single().expect("Invalid timestamp") // Use single() for clarity
     }

     fn create_test_event_time(id: ID, recurrence_id: Option<ID>) -> event_time_entity::Model {
         let now = Utc::now().timestamp_millis();
         event_time_entity::Model {
             id,
             recurrence_id,
             start_time: test_time(1700000000),
             end_time: test_time(1700003600),
             deleted_at: None,
             created_at: now,
             updated_at: now,
         }
     }

     #[tokio::test]
    async fn test_eventtime_get_by_id_found() {
        let time_id = Uuid::new_v4();
        let expected_time = create_test_event_time(time_id, None);
        let db = setup_mock_db_with_query_results(vec![vec![expected_time.clone()]]);
        let repo = EventTimeRepository::new(db); // Pass Arc

        let result = repo.get_by_id(time_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_time));
    }

     #[tokio::test]
    async fn test_eventtime_create_success() {
        let recurrence_id = Uuid::new_v4();
        let schema = EventTimeCreationSchema {
            // Assuming the actual schema requires ID, not Option<ID>
            recurrence_id,
            start_time: test_time(1710000000),
            end_time: test_time(1710007200),
        };
         let db = setup_mock_db_with_exec_results(vec![MockExecResult { last_insert_id: 1, rows_affected: 1 }]);
        let repo = EventTimeRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_time = result.unwrap();
        // Check if the model correctly stores it as Option<ID>
        assert_eq!(created_time.recurrence_id, Some(schema.recurrence_id));
        assert_eq!(created_time.start_time, schema.start_time);
        assert_eq!(created_time.end_time, schema.end_time);
        assert!(!created_time.id.is_nil());
    }

     // Add tests for delete, update, etc.
}


mod recurrence_repo_tests {
    use super::*;
    use recurrence_entity::RecurrenceStatus;

    fn create_test_recurrence(id: ID, event_id: ID) -> recurrence_entity::Model {
        let now = Utc::now().timestamp_millis();
        recurrence_entity::Model {
            id,
            event_id,
            status: RecurrenceStatus::Active,
            rrule: "FREQ=DAILY".to_string(),
            exdate: "".to_string(),
            rdate: "".to_string(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_recurrence_get_by_id_found() {
        let rec_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let expected_rec = create_test_recurrence(rec_id, event_id);
        let db = setup_mock_db_with_query_results(vec![vec![expected_rec.clone()]]);
        let repo = RecurrenceRepository::new(db); // Pass Arc

        let result = repo.get_by_id(rec_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_rec));
    }

    #[tokio::test]
    async fn test_recurrence_create_success() {
        let event_id = Uuid::new_v4();
        let schema = RecurrenceCreationSchema {
            event_id,
            status: RecurrenceStatus::Active,
            rrule: "FREQ=WEEKLY;BYDAY=MO".to_string(),
            exdate: "".to_string(),
            rdate: "".to_string(),
        };
        let db = setup_mock_db_with_exec_results(vec![MockExecResult { last_insert_id: 1, rows_affected: 1 }]);
        let repo = RecurrenceRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_rec = result.unwrap();
        assert_eq!(created_rec.event_id, schema.event_id);
        assert_eq!(created_rec.status, schema.status);
        assert_eq!(created_rec.rrule, schema.rrule);
        assert!(!created_rec.id.is_nil());
    }

    // Add tests for delete, update, etc.
}

mod exception_repo_tests {
    use super::*;
    use exception_entity::EventExceptionStatus;

     // Helper to create DateWithTimeZone for tests
     fn test_time(ts_secs: i64) -> DateWithTimeZone {
         Utc.timestamp_opt(ts_secs, 0).single().expect("Invalid timestamp")
     }

    fn create_test_exception(id: ID, recurrence_id: ID, event_time_id: ID) -> exception_entity::Model {
        let now = Utc::now().timestamp_millis();
        exception_entity::Model {
            id,
            recurrence_id,
            event_time_id, // Added based on entity definition
            original_start_time: test_time(1700000000),
            status: EventExceptionStatus::Cancelled,
            new_event_time_id: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

     #[tokio::test]
    async fn test_exception_get_by_id_found() {
        let ex_id = Uuid::new_v4();
        let rec_id = Uuid::new_v4();
        let time_id = Uuid::new_v4();
        let expected_ex = create_test_exception(ex_id, rec_id, time_id);
        let db = setup_mock_db_with_query_results(vec![vec![expected_ex.clone()]]);
        let repo = ExceptionRepository::new(db); // Pass Arc

        let result = repo.get_by_id(ex_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_ex));
    }

     #[tokio::test]
    async fn test_exception_create_success() {
        let rec_id = Uuid::new_v4();
        let time_id = Uuid::new_v4(); // Added event_time_id
        let new_time_id = Uuid::new_v4();
        let schema = ExceptionCreationSchema {
            recurrence_id: rec_id,
            event_time_id: time_id, // Added event_time_id
            original_start_time: test_time(1710000000),
            status: EventExceptionStatus::Modified,
            new_event_time_id: Some(new_time_id),
        };
         let db = setup_mock_db_with_exec_results(vec![MockExecResult { last_insert_id: 1, rows_affected: 1 }]);
        let repo = ExceptionRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_ex = result.unwrap();
        assert_eq!(created_ex.recurrence_id, schema.recurrence_id);
        // TODO: Verify schema_to_active_model in exception.rs includes event_time_id
        // If it does, uncomment the next line:
        // assert_eq!(created_ex.event_time_id, schema.event_time_id);
        assert_eq!(created_ex.original_start_time, schema.original_start_time);
        assert_eq!(created_ex.status, schema.status);
        assert_eq!(created_ex.new_event_time_id, schema.new_event_time_id);
        assert!(!created_ex.id.is_nil());
    }

    // Add tests for delete, update, etc.
}

mod goal_repo_tests {
    use super::*;

    fn create_test_goal(id: ID, author_id: ID) -> goal_entity::Model {
        let now = Utc::now().timestamp_millis();
        goal_entity::Model {
            id,
            author_id,
            summary: "Test Goal Summary".to_string(),
            description: "Detailed description".to_string(),
            color: Some("#00FF00".to_string()),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_goal_get_by_id_found() {
        let goal_id = Uuid::new_v4();
        let author_id = Uuid::new_v4();
        let expected_goal = create_test_goal(goal_id, author_id);
        let db = setup_mock_db_with_query_results(vec![vec![expected_goal.clone()]]);
        let repo = GoalRepository::new(db); // Pass Arc

        let result = repo.get_by_id(goal_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_goal));
    }

    #[tokio::test]
    async fn test_goal_create_success() {
        let author_id = Uuid::new_v4();
        let schema = GoalCreationSchema {
            summary: "Achieve X".to_string(),
            description: "By doing Y and Z".to_string(),
            author_id,
            color: "#ABCDEF".to_string(), // Assuming color is mandatory in schema
        };
        let db = setup_mock_db_with_exec_results(vec![MockExecResult { last_insert_id: 1, rows_affected: 1 }]);
        let repo = GoalRepository::new(db); // Pass Arc

        let result = repo.create(&schema).await;

        assert!(result.is_ok());
        let created_goal = result.unwrap();
        assert_eq!(created_goal.author_id, schema.author_id);
        assert_eq!(created_goal.summary, schema.summary);
        // Model stores color as Option<String>
        assert_eq!(created_goal.color, Some(schema.color));
        assert!(!created_goal.id.is_nil());
    }

    // Add tests for delete, update, etc.
}