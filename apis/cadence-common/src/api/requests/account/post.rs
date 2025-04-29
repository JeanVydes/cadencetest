use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::error::APIResponseErrorDetail,
    input_validation::{is_valid_country_code, is_valid_email, is_valid_name, meets_password_complexity, PasswordComplexity},
};

use crate::api::requests::traits::Validation;

// --- Account Related Requests ---

/// Represents the data required to create a new Account via email/password registration.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountCreateRequest {
    /// User's primary email address (will be linked to the account).
    #[schema(example = "user@example.com", format = Email)]
    pub email: String,

    /// User's desired password.
    /// Should meet complexity requirements enforced by the service.
    #[schema(
        example = "VeryStrongP@ssw0rd!",
        min_length = 8,
        write_only = true // Doesn't show up in response examples
    )]
    pub password: String,

    /// Confirmation of the user's password.
    /// Must match the `password` field. Checked by the service.
    #[schema(example = "VeryStrongP@ssw0rd!", write_only = true)]
    pub password_confirmation: String,

    /// User's display name (optional).
    #[schema(example = "John Doe", nullable = true)]
    pub name: Option<String>,

    /// User's country code (ISO 3166-1 alpha-2).
    #[schema(example = "US", min_length = 2, max_length = 2)]
    pub country_code_id: String,
}

impl Validation<()> for AccountCreateRequest {
    /// Validates the AccountCreateRequest data.
    /// Returns an error containing *all* validation failures.
    fn validate(&self) -> Result<(), Vec<APIResponseErrorDetail>> {
        let mut details = Vec::new(); // Collect all errors

        if !is_valid_email(&self.email) {
            details.push(APIResponseErrorDetail::body(
                "email",
                "Must be a valid email address.".to_string(),
            ));
        }

        // Password Checks
        if !meets_password_complexity(&self.password, PasswordComplexity::Normal) {
            details.push(APIResponseErrorDetail::body(
                "password",
                "Password doesn't meet password complexity"
            ))
        }

        // Check confirmation regardless of complexity, but maybe pointless if length failed
        if (self.password != self.password_confirmation) && (self.password.len() != self.password_confirmation.len()) {
                details.push(APIResponseErrorDetail::body(
                    "password_confirmation",
                    "Password confirmation does not match.".to_string(),
                ));
        }

        // Name Check
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                // In DB name is optional, keeps this for now
                // Prevent empty/whitespace-only names
                details.push(APIResponseErrorDetail::body(
                    "name",
                    "Name cannot be empty.".to_string(),
                ));
            } else if name.len() > 50 {
                details.push(APIResponseErrorDetail::body(
                    "name",
                    "Name must be at most 50 characters long.".to_string(),
                ));
            }
        }

        // Country Code Checks
        if !is_valid_country_code(&self.country_code_id) {
            details.push(APIResponseErrorDetail::body(
                "country_code",
                "Country code must be exactly 2 uppercase alphabetic characters.".to_string(),
            ));
        }

        // --- Return collected errors if any ---
        if !details.is_empty() {
            return Err(details);
        }
        Ok(())
    }
}

/// Represents the data required to update an Account.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountUpdateRequest {
    #[schema(example = "1234567890abcdef1234567890abcdef")]
    pub id: String, // Account ID
    /// New display name (optional). Send null to clear, omit to keep unchanged.
    #[schema(example = "Johnathan Doe", nullable = true)]
    pub name: Option<Option<String>>, // Option<Option<>> allows explicit null vs unchanged

    /// New country code (optional).
    #[schema(example = "CA", min_length = 2, max_length = 2, nullable = true)]
    pub country_code: Option<String>,

    #[schema(
        example = "VeryStrongP@ssw0rd!",
        min_length = 8,
        write_only = true,
        nullable = true
    )]
    pub password: Option<String>, // Optional password change
    #[schema(example = "VeryStrongP@ssw0rd!", write_only = true, nullable = true)]
    pub password_confirmation: Option<String>, // Optional password confirmation
}

impl Validation<(uuid::Uuid, Option<uuid::Uuid>)> for AccountUpdateRequest {
    fn validate(&self) -> Result<(uuid::Uuid, Option<uuid::Uuid>), Vec<APIResponseErrorDetail>> {
        let mut details = Vec::new();

        // Validate UUID format
        let id = crate::input_validation::string_to_uuid(&self.id);

        if let Err(_) = id {
            details.push(APIResponseErrorDetail::body(
                "id",
                "Must be a valid UUID.".to_string(),
            ));
        }

        // Name Check
        if let Some(ref name) = self.name {
            if let Some(name) = name {
                if !is_valid_name(name) {
                    details.push(APIResponseErrorDetail::body(
                        "name",
                        "Name must be at most 50 characters long.".to_string(),
                    ));
                }
            }
        }

        let mut country_code_id: Option<uuid::Uuid> = None;
        // Country Code Check
        if let Some(ref country_code) = self.country_code {
            match crate::input_validation::string_to_uuid(&country_code) {
                Ok(uuid) => country_code_id = Some(uuid),
                Err(_) => {
                    details.push(APIResponseErrorDetail::body(
                        "country_code",
                        "Country code must be an id.".to_string(),
                    ));
                }
            }
        }

        if let Some(ref password) = self.password {
            if !meets_password_complexity(password, PasswordComplexity::Normal) {
                details.push(APIResponseErrorDetail::body(
                    "password",
                    "Password does not meet complexity requirements.".to_string(),
                ));
            }

            if let Some(ref password_confirmation) = self.password_confirmation {
                if password != password_confirmation {
                    details.push(APIResponseErrorDetail::body(
                        "password_confirmation",
                        "Password confirmation does not match.".to_string(),
                    ));
                }
            } else {
                details.push(APIResponseErrorDetail::body(
                    "password_confirmation",
                    "Password confirmation is required.".to_string(),
                ));
            }
        }

        if !details.is_empty() {
            return Err(details);
        }
        Ok((id.unwrap(), country_code_id))
    }
}

/// Represents the data to add a new email to an existing account.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddEmailRequest {
    #[schema(example = "new_email@example.com", format = Email)]
    pub email: String,
    #[schema(example = false)]
    pub set_as_primary: Option<bool>, // Default to false if omitted
}

impl Validation<()> for AddEmailRequest {
    fn validate(&self) -> Result<(), Vec<APIResponseErrorDetail>> {
        if !is_valid_email(&self.email) {
            return Err(vec![APIResponseErrorDetail::body(
                "email",
                "Must be a valid email address.".to_string(),
            )]);
        }
        Ok(())
    }
}
