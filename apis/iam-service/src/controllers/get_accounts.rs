use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use cadence_common::api::{
    error::{APIResponseError, APIResponseErrorDetail},
    requests::{account::get::GetAccountsQuery, traits::Validation},
    response::{APIResponse, APIResponseObjectType},
    state::ApplicationState,
};
use cadence_common::repository_traits::CrudEntityRepository;
use serde_json::Value;

use crate::{
    controllers::common::CensoredAccountResponse,
    responses::{failed_to_x_account, invalid_input},
    service::ServiceState,
};

#[utoipa::path(
    get,
    path = "/accounts",
    params(GetAccountsQuery),
     responses(
        (status = 200, description = "Accounts retrieved successfully", body = APIResponse<Vec<CensoredAccountResponse>>),
        (status = 400, description = "Invalid input / Validation Error", body = APIResponse<Value>, example = json!(
            invalid_input("query_params", vec![APIResponseErrorDetail::query("ids", "Each ID must be a valid UUID.")])
        )),
        (status = 500, description = "Internal Server Error", body = APIResponse<Value>, example = json!(
            failed_to_x_account("retrieve")
        ))
    ),
    tag = "Account"
)]
#[axum::debug_handler]
pub async fn get_accounts_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    Query(query): Query<GetAccountsQuery>,
) -> Result<impl IntoResponse, APIResponseError> {
    let accounts_id = query
        .validate()
        .map_err(|details| invalid_input("query_params", details))?;

    let accounts = state
        .services
        .account_service
        .account_repository
        .get_by_ids(accounts_id)
        .await
        .map_err(|_| failed_to_x_account("retrieve"))?;

    let response_dto = accounts
        .into_iter()
        .map(|account_model| CensoredAccountResponse::from(account_model))
        .collect::<Vec<CensoredAccountResponse>>();

    Ok(APIResponse::success(
        response_dto,
        APIResponseObjectType::Account,
    ))
}
