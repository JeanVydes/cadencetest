use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use cadence_common::api::{
    error::{APIResponseError, APIResponseErrorDetail},
    requests::account::get::GetAccountQuery,
    response::{APIResponse, APIResponseObjectType},
    state::ApplicationState,
};
use cadence_common::repository_traits::CrudEntityRepository;
use serde_json::Value;

use crate::{
    controllers::common::CensoredAccountResponse,
    responses::{failed_to_x_account, invalid_input, not_found_entity},
    service::ServiceState,
};
use cadence_common::api::requests::traits::Validation;

#[utoipa::path(
    get,
    path = "/account",
    params(GetAccountQuery),
     responses(
        (status = 200, description = "Account retrieved successfully", body = APIResponse<CensoredAccountResponse>),
        (status = 400, description = "Invalid input / Validation Error", body = APIResponse<Value>, example = json!(
            invalid_input("query_params", vec![APIResponseErrorDetail::query("id", "Must be a valid UUID.")])
        )),
        (status = 404, description = "Account not found", body = APIResponse<Value>, example = json!(
            not_found_entity("account")
        )),
        (status = 500, description = "Internal Server Error", body = APIResponse<Value>, example = json!(
            failed_to_x_account("retrieve")
        ))
    ),
    tag = "Account"
)]
#[axum::debug_handler]
pub async fn get_account_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    Query(query): Query<GetAccountQuery>,
) -> Result<impl IntoResponse, APIResponseError> {
    let account_id = query
        .validate()
        .map_err(|details| invalid_input("query_params", details))?;

    let account = state
        .services
        .account_service
        .account_repository
        .get_by_id(account_id)
        .await
        .map_err(|_| failed_to_x_account("retrieve"))?
        .ok_or_else(|| not_found_entity("account"))?;

    Ok(APIResponse::<CensoredAccountResponse>::success(
        CensoredAccountResponse::from(account),
        APIResponseObjectType::Account,
    ))
}
