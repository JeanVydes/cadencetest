use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{AuthError, CadenceError, DatabaseError, EntityError, InputError, ServerError};

use super::response::APIResponse;

// --- Error Structures ---

/// Represents an error that occurred during API processing.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct APIResponseError {
    /// Structured error code and details.
    pub error: CadenceError,
    /// Human-readable message describing the overall error.
    #[schema(example = "Input validation failed")]
    pub message: String,
    /// List of specific error details (e.g., field validation issues).
    pub details: Vec<APIResponseErrorDetail>,
}

/// Provides specific details about an error, often related to a request field.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct APIResponseErrorDetail {
    /// Detailed feedback on the specific issue.
    #[schema(example = "Field 'email' must be a valid email address.")]
    pub detailed_feedback: String,
    /// Source of the error (e.g., field name in the request body).
    #[schema(nullable = true, example = "body.name")]
    pub source: Option<String>,
}

// --- Implementation block for APIResponseError helpers ---

impl APIResponseError {
    /// Creates a new APIResponseError.
    pub fn new(
        error: CadenceError,
        message: impl Into<String>,
        details: Vec<APIResponseErrorDetail>,
    ) -> Self {
        Self {
            error,
            message: message.into(),
            details,
        }
    }

    /// Helper for input errors.
    pub fn input_error(
        kind: InputError,
        message: impl Into<String>,
        details: Vec<APIResponseErrorDetail>,
    ) -> Self {
        Self::new(CadenceError::Input(kind), message, details)
    }

    /// Helper for auth errors.
    pub fn auth_error(
        kind: AuthError,
        message: impl Into<String>,
        details: Vec<APIResponseErrorDetail>,
    ) -> Self {
        Self::new(CadenceError::Auth(kind), message, details)
    }

    /// Helper for entity errors.
    pub fn entity_error(
        kind: EntityError,
        message: impl Into<String>,
        details: Vec<APIResponseErrorDetail>,
    ) -> Self {
        Self::new(CadenceError::Entity(kind), message, details)
    }

    pub fn db_error(
        kind: DatabaseError,
        message: impl Into<String>,
        details: Vec<APIResponseErrorDetail>,
    ) -> Self {
        Self::new(CadenceError::Database(kind), message, details)
    }

    /// Helper for a common Not Found error.
    pub fn not_found(entity_type: &str, id: &str) -> Self {
        Self::entity_error(
            EntityError::NotFound(format!("{} with ID {} not found", entity_type, id)),
            format!("{} not found", entity_type),
            Vec::new(),
        )
    }

    pub fn server_error(
        kind: ServerError,
        message: impl Into<String>,
        details: Vec<APIResponseErrorDetail>,
    ) -> Self {
        Self::new(CadenceError::ServerError(kind), message, details)
    }

}


impl APIResponseErrorDetail {
    /// Creates a new APIResponseErrorDetail.
    pub fn new(
        feedback: impl Into<String>,
        source: Option<String>,
    ) -> Self {
        Self {
            detailed_feedback: feedback.into(),
            source,
        }
    }

    pub fn body(field: impl Into<String>, feedback: impl Into<String>) -> Self {
        Self::new(feedback, Some(format!("body.{}", field.into())))
    }

    pub fn query(field: impl Into<String>, feedback: impl Into<String>) -> Self {
        Self::new(feedback, Some(format!("query.{}", field.into())))
    }

    pub fn header(field: impl Into<String>, feedback: impl Into<String>) -> Self {
        Self::new(feedback, Some(format!("header.{}", field.into())))
    }

    pub fn path(field: impl Into<String>, feedback: impl Into<String>) -> Self {
        Self::new(feedback, Some(format!("path.{}", field.into())))
    }
}

impl IntoResponse for APIResponseError {
    fn into_response(self) -> Response {
        // 1. Determine the appropriate HTTP Status Code based on the error type
        let status_code = match &self.error {
            CadenceError::Input(_) => {
                // Input errors are typically client errors
                StatusCode::BAD_REQUEST // 400
                // Could potentially map specific InputErrors to 422 Unprocessable Entity
            }
            CadenceError::Auth(auth_error) => {
                // Auth errors map to 401 or 403
                match auth_error {
                    AuthError::Unauthorized(_) => StatusCode::UNAUTHORIZED, // 401
                    AuthError::InvalidCredentials(_) => StatusCode::UNAUTHORIZED, // 401
                    AuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED, // 401
                    AuthError::InvalidScope(_) => StatusCode::FORBIDDEN, // 403 (Has credentials, but not allowed)
                    _ => StatusCode::UNAUTHORIZED, // Default for other auth issues
                }
            }
            CadenceError::Entity(entity_error) => {
                // Entity errors can map to various codes
                match entity_error {
                    EntityError::NotFound(_) => StatusCode::NOT_FOUND, // 404
                    EntityError::AlreadyExists(_) => StatusCode::CONFLICT, // 409
                    EntityError::InvalidState(_) => StatusCode::BAD_REQUEST, // 400 (or 409 Conflict sometimes)
                    EntityError::InvalidTransition(_) => StatusCode::BAD_REQUEST, // 400
                    EntityError::InvalidUniqueConstraint(_) => StatusCode::CONFLICT, // 409
                    // Consider other specific mappings
                    _ => StatusCode::INTERNAL_SERVER_ERROR, // Default for unexpected entity/db issues
                }
            }
            CadenceError::Database(_) => {
                StatusCode::INTERNAL_SERVER_ERROR // 500
            }
            CadenceError::ServerError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR // 500
            }
        };

        let response_wrapper = APIResponse::<()>::failure(
            self,
            status_code,
        );

        match serde_json::to_string(&response_wrapper) {
            Ok(json_body) => Response::builder()
                .status(status_code)
                .header(header::CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(json_body))
                .unwrap_or_else(|_| {
                    tracing::error!("Failed to build response body after serialization.");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(axum::body::Body::from("Internal Server Error"))
                        .unwrap()
                }),
            Err(e) => {
                tracing::error!("Failed to serialize APIResponseError: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(
                        r#"{"metadata":{"status":"failure"},"errors":{"message":"Internal server error during error serialization"}}"#,
                    ))
                    .unwrap()
            }
        }
    }
}
