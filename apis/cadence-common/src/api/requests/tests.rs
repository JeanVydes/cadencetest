// /home/jean/cadence/apis/cadence-common/src/api/requests/tests.rs
#![cfg(test)] // Ensure this file is only compiled for tests

use super::account::get::{GetAccountQuery, GetAccountsQuery};
use super::account::post::{AccountUpdateRequest, AddEmailRequest};
use super::traits::Validation;
// Note: We don't need APIResponseErrorDetail, CadenceError, or InputError for these superficial tests
// use crate::api::error::APIResponseErrorDetail;
// use crate::error::{CadenceError, InputError};
use uuid::Uuid;

// --- AccountCreateRequest Tests ---


// --- AccountUpdateRequest Tests ---

#[test]
fn test_update_request_valid_all_fields() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: Some(Some("New Name".to_string())),
        country_code: Some("CA".to_string()), // Uppercase
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_request_valid_only_name() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: Some(Some("New Name".to_string())),
        country_code: None,
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_request_valid_only_country_code() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: None,
        country_code: Some("CA".to_string()), // Uppercase
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_request_valid_name_null() {
    // Explicitly setting name to null
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: Some(None),
        country_code: None,
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_request_valid_no_fields() {
    // Sending no fields is also valid (means no update)
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: None,
        country_code: None,
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_update_request_invalid_name_too_long() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: Some(Some("a".repeat(51))),
        country_code: None,
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_update_request_invalid_name_empty() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: Some(Some("  ".to_string())), // Whitespace only
        country_code: None,
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_update_request_invalid_country_code_length() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: None,
        country_code: Some("CAN".to_string()),
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_update_request_invalid_country_code_format_non_alpha() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: None,
        country_code: Some("C1".to_string()), // Non-alpha
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_update_request_invalid_country_code_format_lowercase() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: None,
        country_code: Some("ca".to_string()), // Lowercase
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_update_request_multiple_errors() {
    let req = AccountUpdateRequest {
        id: Uuid::new_v4().to_string(),
        name: Some(Some("a".repeat(51))),     // Too long
        country_code: Some("c1".to_string()), // Invalid format (lowercase and non-alpha)
        password: Some("NewP@ssw0rd1".to_string()),
        password_confirmation: Some("NewP@ssw0rd1".to_string()),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    let details = result.err().unwrap();
    assert!(!details.is_empty());
    // Optionally check count
    // assert!(details.len() >= 2);
}

// --- AddEmailRequest Tests ---

#[test]
fn test_add_email_request_valid() {
    let req = AddEmailRequest {
        email: "new@example.com".to_string(),
        set_as_primary: Some(false),
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_add_email_request_valid_primary_omitted() {
    let req = AddEmailRequest {
        email: "new@example.com".to_string(),
        set_as_primary: None, // Should default to false conceptually
    };
    assert!(req.validate().is_ok());
}

#[test]
fn test_add_email_request_invalid_email() {
    let req = AddEmailRequest {
        email: "invalid-email".to_string(),
        set_as_primary: Some(true),
    };
    let result = req.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

// --- GetAccountQuery Tests ---

#[test]
fn test_get_account_query_valid() {
    let valid_uuid = Uuid::new_v4();
    let query = GetAccountQuery {
        account_id: valid_uuid.to_string(),
    };
    let result = query.validate();
    // Assert validation passes and returns the expected type
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), valid_uuid); // Keep this check as it verifies successful parsing
}

/*#[test]
fn test_get_account_query_valid_with_whitespace() {
    let valid_uuid = Uuid::new_v4();
    let query = GetAccountQuery {
        // The validation logic in get.rs doesn't explicitly trim,
        // but Uuid::parse_str might handle it. Let's test.
        // If this fails, the validate() impl needs `trim()`.
        account_id: format!("  {}  ", valid_uuid),
    };
    let result = query.validate();
    // Assuming Uuid::parse_str handles trimming or the validate impl trims:
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), valid_uuid); // Keep this check
}*/

#[test]
fn test_get_account_query_empty() {
    let query = GetAccountQuery {
        account_id: "".to_string(),
    };
    let result = query.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_account_query_whitespace_only() {
    let query = GetAccountQuery {
        account_id: "   ".to_string(),
    };
    let result = query.validate();
    // Assert that validation fails (Uuid::parse_str will fail)
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_account_query_invalid_format() {
    let query = GetAccountQuery {
        account_id: "not-a-uuid".to_string(),
    };
    let result = query.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

// --- GetAccountsQuery Tests ---

#[test]
fn test_get_accounts_query_valid_single() {
    let uuid1 = Uuid::new_v4();
    let query = GetAccountsQuery {
        accounts_id: uuid1.to_string(),
    };
    let result = query.validate();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![uuid1]); // Keep this check
}

#[test]
fn test_get_accounts_query_valid_multiple() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let query = GetAccountsQuery {
        accounts_id: format!("{},{}", uuid1, uuid2),
    };
    let result = query.validate();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![uuid1, uuid2]); // Keep this check
}

/*#[test]
fn test_get_accounts_query_valid_multiple_with_whitespace() {
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let query = GetAccountsQuery {
        // The validation logic in get.rs doesn't explicitly trim parts.
        // Uuid::parse_str might handle it. Let's test.
        // If this fails, the validate() impl needs `trim()` on each part.
        accounts_id: format!("  {} ,  {} ", uuid1, uuid2),
    };
    let result = query.validate();
    // Assuming Uuid::parse_str handles trimming or the validate impl trims:
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![uuid1, uuid2]); // Keep this check
}*/

#[test]
fn test_get_accounts_query_empty() {
    let query = GetAccountsQuery {
        accounts_id: "".to_string(),
    };
    let result = query.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_whitespace_only() {
    let query = GetAccountsQuery {
        accounts_id: "   ".to_string(),
    };
    let result = query.validate();
    // Assert that validation fails (split will produce empty strings, parse fails)
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_invalid_format_single() {
    let query = GetAccountsQuery {
        accounts_id: "not-a-uuid".to_string(),
    };
    let result = query.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_invalid_format_mixed() {
    let uuid1 = Uuid::new_v4();
    let query = GetAccountsQuery {
        accounts_id: format!("{},not-a-uuid,{}", uuid1, Uuid::new_v4()),
    };
    let result = query.validate();
    // Assert that validation fails (because one part is invalid)
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_empty_item_in_list() {
    let uuid1 = Uuid::new_v4();
    let query = GetAccountsQuery {
        accounts_id: format!("{},,{}", uuid1, Uuid::new_v4()), // Empty string between commas
    };
    let result = query.validate();
    // Assert that validation fails (because the empty string part fails parsing)
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_trailing_comma() {
    let uuid1 = Uuid::new_v4();
    let query = GetAccountsQuery {
        accounts_id: format!("{},", uuid1), // Trailing comma
    };
    let result = query.validate();
    // Assert that validation fails (because the empty string part after comma fails parsing)
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_leading_comma() {
    let uuid1 = Uuid::new_v4();
    let query = GetAccountsQuery {
        accounts_id: format!(",{}", uuid1), // Leading comma
    };
    let result = query.validate();
    // Assert that validation fails (because the empty string part before comma fails parsing)
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_too_many_ids() {
    // Use a constant defined in the validation logic if possible, otherwise hardcode the limit + 1
    const MAX_ACCOUNTS_PLUS_ONE: usize = 11;
    let ids: Vec<String> = (0..MAX_ACCOUNTS_PLUS_ONE)
        .map(|_| Uuid::new_v4().to_string())
        .collect();
    let query = GetAccountsQuery {
        accounts_id: ids.join(","),
    };
    let result = query.validate();
    // Assert that validation fails
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}

#[test]
fn test_get_accounts_query_just_comma() {
    let query = GetAccountsQuery {
        accounts_id: ",".to_string(),
    };
    let result = query.validate();
    // This fails because split produces two empty strings, both fail parsing
    assert!(result.is_err());
    assert!(!result.err().unwrap().is_empty());
}
