
use std::{fs::File, io::Write};

use utoipa::{openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme}, Modify, OpenApi};

// --- Common API Imports ---
use cadence_common::{api::{
    // Errors
    error::{
        APIResponseError, APIResponseErrorDetail,
    },
    // Requests (Payloads & Query Params)
    requests::account::{
            get::{GetAccountQuery, GetAccountsQuery}, post::{AccountCreateRequest, AccountUpdateRequest} // GET Query Params
        },
    // Generic Response Wrapper & Metadata
    response::{APIResponse, APIResponseMetadata, APIResponseObjectType, APIResponseStatus},
}, error::{AuthError, CadenceError, DatabaseError, EntityError, InputError, ServerError}};

// --- Service-Specific Imports ---
// Import the specific DTO used in success responses
use iam_service::controllers::common::CensoredAccountResponse; // This should be the actual DTO used in your success responses

// --- Security Modifier ---
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth", // The name used in #[utoipa::path(security(...))]
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT") // Optional: Specify the format
                        .description(Some("Bearer token authentication using JWT.".to_string()))
                        .build(),
                ),
            )
        }
    }
}


#[derive(OpenApi)]
#[openapi(
    // --- Paths ---
    // Include all controller functions that define API endpoints
    paths(
        iam_service::controllers::create_account::create_account_controller,
        iam_service::controllers::get_account::get_account_controller,
        iam_service::controllers::get_accounts::get_accounts_controller, // Added
        iam_service::controllers::update_account::update_account_controller, // Added
        // Add other controller paths here as needed
        // iam_service::controllers::login::login_controller,
        // iam_service::controllers::add_email::add_email_controller,
    ),
    // --- Components ---
    // Define all data structures used in requests, responses, and errors
    components(
        schemas(
            // == Request Structures ==
            // Payloads
            AccountCreateRequest,
            AccountUpdateRequest,
            // AddEmailRequest, // Keep if used by other endpoints
            // LoginRequest,    // Keep if used by other endpoints
            // Query Parameters
            GetAccountQuery,   // Added
            GetAccountsQuery,  // Added

            // == Response Structures ==
            // Generic Wrapper & Metadata
            APIResponseMetadata,
            APIResponseStatus,
            APIResponseObjectType,
            // Specific Success DTOs
            CensoredAccountResponse, // Added (the actual data structure)

            // == Error Structures ==
            APIResponseError,       // Top-level error wrapper
            APIResponseErrorDetail, // Error detail structure
            // Core Error Enum Hierarchy (used within APIResponseError.error)
            CadenceError,           // Added (replaces APIError)
            InputError,
            AuthError,
            EntityError,
            DatabaseError,          // Added
            ServerError,            // Added

            // == Concrete APIResponse Instances ==
            // Used in success responses (add for each distinct success body type)
            APIResponse<CensoredAccountResponse>,                // Added
            APIResponse<Vec<CensoredAccountResponse>>,           // Added
            // Used in error response examples (or if an endpoint explicitly returns it)
            APIResponse<serde_json::Value>,
            // APIResponse<Value> is often used for examples where the specific success type isn't relevant

            // Remove unused/incorrect ones:
            // APIError, // Replaced by CadenceError
            // AccountResponse, // Replaced by CensoredAccountResponse
            // APIResponse<AccountResponse>, // Replaced by APIResponse<CensoredAccountResponse>
        )
    ),
    // --- Security Definitions ---
    // Define the security schemes referenced in paths
    modifiers(&SecurityAddon), // Added to define "bearer_auth"
    // --- Tags ---
    // Group related endpoints in the UI
    tags(
        (name = "Account", description = "Account management operations (CRUD)"), // Updated description
        // (name = "Authentication", description = "Authentication operations"), // Keep if login endpoint is added
        // (name = "Email", description = "Email management operations"), // Keep if email endpoint is added
    ),
    // --- General API Info ---
    info(
        title = "Cadence IAM Service API", // Made more specific to the service
        version = "0.1.0",
        description = "OpenAPI specification for the Cadence Identity and Access Management (IAM) service.", // Updated description
        contact(
            name = "Cadence Labs",
            // url = Some("https://your-company.com"), // Optional
            // email = Some("dev@your-company.com"), // Optional
        ),
        // license = Some(utoipa::openapi::License::new("MIT")), // Optional
    ),
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Generating OpenAPI specification...");
    generate_openapi_spec()?;
    tracing::info!("OpenAPI specification generation complete.");

    Ok(())
}

fn generate_openapi_spec() -> std::io::Result<()> {
    let openapi_spec = ApiDoc::openapi();
    let json_spec = openapi_spec
        .to_pretty_json()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to serialize OpenAPI spec to JSON: {}", e)))?; // Improved error handling

    let output_filename = "openapi.json"; // Consider making this configurable
    tracing::info!("Writing OpenAPI specification to '{}'", output_filename);
    let mut file = File::create(output_filename)?;
    file.write_all(json_spec.as_bytes())?;
    tracing::info!(
        "✅ OpenAPI specification generated successfully at '{}'",
        output_filename
    );
    Ok(())
}
