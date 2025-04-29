use std::sync::Arc;

use crate::middlewares::auth::Authenticated;
use crate::responses::{
    delegated_account_dont_match, error_hashing_password, failed_to_x_account, invalid_input,
    not_found_entity,
};
use crate::service::ServiceState;

use super::common::CensoredAccountResponse;
use axum::{extract::State, response::IntoResponse};
use cadence_common::api::axum_rejections::CadenceJsonExtractor;
use cadence_common::api::requests::account::post::AccountUpdateRequest;
use cadence_common::api::requests::traits::Validation;
use cadence_common::entities::services::account::AccountServiceUpdateSchema;
use cadence_common::{
    api::{error::APIResponseError, response::APIResponse, state::ApplicationState},
    input_validation::password_to_hashed,
};
use serde_json::Value;

#[utoipa::path(
    patch,
    path = "/account",
    request_body = AccountUpdateRequest,
    security(
        ("bearer_auth" = [])
    ),
     responses(
        (status = 200, description = "Account updated successfully", body = APIResponse<CensoredAccountResponse>),
        (status = 400, description = "Invalid input / Validation Error", body = APIResponse<Value>, example = json!(invalid_input("body", vec![]))),
        (status = 403, description = "Forbidden - Cannot update another user's account", body = APIResponse<Value>, example = json!(delegated_account_dont_match())),
        (status = 404, description = "Account not found", body = APIResponse<Value>, example = json!(not_found_entity("account"))),
        (status = 500, description = "Internal Server Error", body = APIResponse<Value>, example = json!(failed_to_x_account("update")))
    ),
    tag = "Account"
)]
#[axum::debug_handler]
pub async fn update_account_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    Authenticated(claims): Authenticated,
    CadenceJsonExtractor(payload): CadenceJsonExtractor<AccountUpdateRequest>,
) -> Result<impl IntoResponse, APIResponseError> {
    let (id, country_code_id) = payload
        .validate()
        .map_err(|details| invalid_input("body", details))?;

    if claims.sub != id {
        return Err(delegated_account_dont_match());
    }

    let mut password: Option<String> = None;
    if let Some(new_password) = payload.password {
        password = Some(password_to_hashed(&new_password).map_err(|_| error_hashing_password())?);
    }

    let mut schema: AccountServiceUpdateSchema = AccountServiceUpdateSchema {
        name: None,
        country_code_id,
        password,
    };

    if payload.name.is_some() {
        schema.name = payload.name.unwrap();
    }

    let account = state
        .services
        .account_service
        .update(id, schema)
        .await
        .map_err(|_| failed_to_x_account("update"))?;

    Ok(APIResponse::<CensoredAccountResponse>::success(
        CensoredAccountResponse::from(account),
        cadence_common::api::response::APIResponseObjectType::Account,
    ))
}
