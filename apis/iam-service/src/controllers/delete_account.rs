use std::sync::Arc;

use crate::middlewares::auth::Authenticated;
use crate::responses::{
    delegated_account_dont_match, failed_to_x_account, invalid_input, not_found_entity,
};
use crate::service::ServiceState;

use super::common::CensoredAccountResponse;
use axum::{extract::State, response::IntoResponse};
use cadence_common::api::requests::account::post::AccountUpdateRequest;
use cadence_common::api::{
    error::APIResponseError, response::APIResponse, state::ApplicationState,
};
use cadence_common::repository_traits::CrudEntityRepository;
use serde_json::Value;

#[utoipa::path(
    delete,
    path = "/account",
    request_body = AccountUpdateRequest,
    security(
        ("bearer_auth" = [])
    ),
     responses(
        (status = 200, description = "Account deleted successfully", body = APIResponse<Value>),
        (status = 400, description = "Invalid input / Validation Error", body = APIResponse<Value>, example = json!(invalid_input("body", vec![]))),
        (status = 403, description = "Forbidden - Cannot update another user's account", body = APIResponse<Value>, example = json!(delegated_account_dont_match())),
        (status = 404, description = "Account not found", body = APIResponse<Value>, example = json!(not_found_entity("account"))),
        (status = 500, description = "Internal Server Error", body = APIResponse<Value>, example = json!(failed_to_x_account("update")))
    ),
    tag = "Account"
)]
#[axum::debug_handler]
pub async fn delete_account_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    Authenticated(claims): Authenticated,
) -> Result<impl IntoResponse, APIResponseError> {
    let account = state
        .services
        .account_service
        .account_repository
        .delete(claims.sub)
        .await
        .map_err(|_| failed_to_x_account("delete"))?;

    Ok(APIResponse::<CensoredAccountResponse>::success(
        CensoredAccountResponse::from(account),
        cadence_common::api::response::APIResponseObjectType::Account,
    ))
}
