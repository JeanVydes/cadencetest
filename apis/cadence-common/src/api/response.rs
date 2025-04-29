use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use super::error::APIResponseError; // Assuming ID = Uuid, ensure crate is added

// --- Main Response Structure ---

/// Generic structure for all API responses.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct APIResponse<T: Serialize + ToSchema + 'static> {
    /// Metadata about the request and response.
    pub metadata: APIResponseMetadata,
    /// The requested data payload (if successful and applicable).
    #[schema(nullable = true)]
    pub data: Option<T>,
    /// Error details if the operation failed.
    #[schema(nullable = true)]
    pub errors: Option<APIResponseError>,
}

impl <T>IntoResponse for APIResponse<T> 
where
    T: Serialize + ToSchema + 'static,
{
    fn into_response(self) -> axum::response::Response {
        let status_code = StatusCode::from_u16(self.metadata.http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        (status_code, body).into_response()
    }
}

// --- Metadata ---

/// Metadata about the API response.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct APIResponseMetadata {
    /// API version used (e.g., "v1.0").
    #[schema(example = "v1.0")]
    pub api_version: String,
    /// General status of the response.
    #[schema(example = json!(APIResponseStatus::Success))]
    pub status: APIResponseStatus,
    pub http_status: u16,
    /// UTC timestamp of response generation (ISO 8601).
    #[schema(value_type = String, format = DateTime, example = "2023-10-01T12:00:00Z")]
    pub timestamp: DateTime<Utc>,
    /// Primary object type contained in the `data` field.
    #[schema(example = json!(APIResponseObjectType::Account))]
    pub data_type: APIResponseObjectType,
}

// --- Enums ---

/// General status of the API response.
#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum APIResponseStatus {
    Success,
    Failure,
}

/// Primary object types that can be returned in the API response `data` field.
#[derive(Debug, Clone, Deserialize, Serialize, EnumIter, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum APIResponseObjectType {
    Account,
    EventMetadata,
    EventTime,
    Recurrence,
    Exception,
    Goal,
    Tag,
    Flag,
    Email,
    ExternalIdentity,
    Energy,
    Unknown,
    Auth,
    None,
}

// --- Implementation block for APIResponse constructors ---

const CURRENT_API_VERSION: &str = "v1.0"; // Or load from config

impl<T> APIResponse<T>
where
    T: Serialize + ToSchema + 'static,
{
    /// Creates a successful API response containing data.
    pub fn success(data: T, data_type: APIResponseObjectType) -> Self {
        let metadata = APIResponseMetadata {
            api_version: CURRENT_API_VERSION.to_string(),
            status: APIResponseStatus::Success,
            http_status: 200,
            timestamp: Utc::now(),
            data_type,
        };
        Self { metadata, data: Some(data), errors: None }
    }

    /// Creates a successful API response with no data payload.
    pub fn success_no_data() -> APIResponse<()> {
        let metadata = APIResponseMetadata {
            api_version: CURRENT_API_VERSION.to_string(),
            status: APIResponseStatus::Success,
            http_status: 200,
            timestamp: Utc::now(),
            data_type: APIResponseObjectType::None,
        };
        APIResponse::<()> { metadata, data: None, errors: None }
    }

    /// Creates a failure API response containing error details.
    pub fn failure(error: APIResponseError, http_status: StatusCode) -> Self {
        let metadata = APIResponseMetadata {
            api_version: CURRENT_API_VERSION.to_string(),
            status: APIResponseStatus::Failure,
            http_status: http_status.as_u16(),
            timestamp: Utc::now(),
            data_type: APIResponseObjectType::Unknown, // Or None? Or infer from error somehow? Unknown is safer.
        };
        Self { metadata, data: None, errors: Some(error) }
    }
}