use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::error::APIResponseErrorDetail,
    input_validation::{is_valid_email, meets_password_complexity},
};

use crate::api::requests::traits::Validation;

// --- Auth Related Requests --
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ObtainTokenRequest {
    /// User's email_address
    #[schema(example = "user@example.com", format = Email)]
    pub email: String,

    /// Should meet complexity requirements enforced by the service.
    #[schema(
        example = "VeryStrongP@ssw0rd!",
        min_length = 8,
        write_only = true // Doesn't show up in response examples
    )]
    pub password: String,
}

impl Validation<()> for ObtainTokenRequest {
    fn validate(&self) -> Result<(), Vec<APIResponseErrorDetail>> {
        let mut details = Vec::new(); // Collect all errors

        if !is_valid_email(&self.email) {
            details.push(APIResponseErrorDetail::body(
                "email",
                "Must be a valid email address.".to_string(),
            ));
        }

        // Password Checks
        if !meets_password_complexity(
            &self.password,
            crate::input_validation::PasswordComplexity::Normal,
        ) {
            details.push(APIResponseErrorDetail::body(
                "password",
                "Password does not meet complexity requirements.".to_string(),
            ));
        }

        // --- Return collected errors if any ---
        if !details.is_empty() {
            return Err(details);
        }

        Ok(())
    }
}