use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::api::error::APIResponseErrorDetail;
use crate::api::requests::traits::Validation;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetAccountQuery {
    #[serde(rename = "id")]
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub account_id: String,
}

impl Validation<uuid::Uuid> for GetAccountQuery {
    fn validate(&self) -> Result<uuid::Uuid, Vec<APIResponseErrorDetail>> {
        let mut details = Vec::new();

        if self.account_id.is_empty() {
            details.push(APIResponseErrorDetail::body(
                "id",
                "Account ID cannot be empty.".to_string(),
            ));
        }

        let uuid = uuid::Uuid::parse_str(&self.account_id).map_err(|_| {
            details.push(APIResponseErrorDetail::body(
                "id",
                format!("Invalid account ID format: {}", self.account_id),
            ));
            details
        })?;

        Ok(uuid)
    }
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetAccountsQuery {
    #[serde(rename = "id")]
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub accounts_id: String,
}

impl Validation<Vec<uuid::Uuid>> for GetAccountsQuery {
    fn validate(&self) -> Result<Vec<uuid::Uuid>, Vec<APIResponseErrorDetail>> {
        let mut details = Vec::new();

        if self.accounts_id.is_empty() {
            details.push(APIResponseErrorDetail::body(
                "id",
                "Account ID cannot be empty.".to_string(),
            ));
        }

        let mut ids = Vec::new();

        for id in self.accounts_id.split(',') {
            if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                ids.push(uuid);
            } else {
                details.push(APIResponseErrorDetail::body(
                    "id",
                    format!("Invalid account ID format: {}", id),
                ));
            }
        }

        if ids.is_empty() {
            details.push(APIResponseErrorDetail::body(
                "id",
                "At least one account ID must be provided.".to_string(),
            ));
        }

        // Don't retrieve more than 10 accounts at once
        if ids.len() > 10 {
            details.push(APIResponseErrorDetail::body(
                "id",
                "Cannot retrieve more than 10 accounts at once.".to_string(),
            ));
        }

        if !details.is_empty() {
            return Err(details);
        }

        Ok(ids)
    }
}
