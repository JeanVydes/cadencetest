use std::sync::Arc;

use crate::responses::{entity_already_exists, error_hashing_password, failed_to_x_account, invalid_input};
use crate::service::ServiceState;

use super::common::CensoredAccountResponse;
use axum::{extract::State, response::IntoResponse};
use cadence_common::api::axum_rejections::CadenceJsonExtractor;
use cadence_common::api::requests::traits::Validation;
use cadence_common::{
    api::{
        error::{APIResponseError, APIResponseErrorDetail},
        requests::account::post::AccountCreateRequest,
        response::APIResponse,
        state::ApplicationState,
    },
    entities::services::account::AccountServiceCreationSchema,
    input_validation::password_to_hashed,
};
use serde_json::Value;

#[utoipa::path(
    post,
    path = "/account",
    request_body = AccountCreateRequest,
     responses(
        (status = 201, description = "Account created successfully", body = APIResponse<CensoredAccountResponse>),
        (status = 400, description = "Invalid input / Validation Error", body = APIResponse<Value>, example = json!(invalid_input("body", vec![APIResponseErrorDetail::body("email", "Must be a valid email address.")]))),
        (status = 409, description = "Conflict (e.g., email already exists)", body = APIResponse<Value>, example = json!(entity_already_exists("Account", "email", "jean@example.com"))),
        (status = 500, description = "Internal Server Error", body = APIResponse<Value>, example = json!(failed_to_x_account("create")))
    ),
    tag = "Account"
)]
#[axum::debug_handler]
pub async fn create_account_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    CadenceJsonExtractor(payload): CadenceJsonExtractor<AccountCreateRequest>,
) -> Result<impl IntoResponse, APIResponseError> {
    payload
        .validate()
        .map_err(|details| invalid_input("body", details))?;

    let password = password_to_hashed(&payload.password).map_err(|_| error_hashing_password())?;
    let country_code_id = uuid::Uuid::parse_str(&payload.country_code_id)
        .map_err(|_| invalid_input("body.country_code_id", vec![APIResponseErrorDetail::body(
            "country_code_id",
            "Must be a valid UUID.".to_string(),
        )]))?;

    let mut schema: AccountServiceCreationSchema = AccountServiceCreationSchema {
        account: cadence_common::entities::account::repositories::account::CreationSchema {
            name: payload.name,
            country_code_id,
            password,
        },
        emails: Vec::new(),
    };

    schema.emails.push(
        cadence_common::entities::account::repositories::email::CreationSchema {
            email: payload.email,
            primary: true,
            verification_code: None,
        },
    );

    let (account, _) = state
        .services
        .account_service
        .create_with_emails(schema)
        .await
        .map_err(|_| failed_to_x_account("create"))?;

    Ok(APIResponse::<CensoredAccountResponse>::success(
        CensoredAccountResponse::from(account),
        cadence_common::api::response::APIResponseObjectType::Account,
    ))
}
