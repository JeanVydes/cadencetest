use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use cadence_common::api::axum_rejections::CadenceJsonExtractor;
use cadence_common::api::requests::auth::post::ObtainTokenRequest;
use cadence_common::api::requests::traits::Validation;
use cadence_common::api::service::service::EnviromentCommon;
use cadence_common::api::{
    error::APIResponseError, response::APIResponse, state::ApplicationState,
};
use cadence_common::input_validation::check_password;
use cadence_common::time::now_millis;
use cadence_common::token::token::{Claims, Scope, TokenType};
use serde::Serialize;
use tracing::trace;
use utoipa::ToSchema;

use crate::responses::{
    error_hashing_password, error_issueing_token, failed_to_x_account, invalid_input,
    invalid_password, not_found_entity,
};
use crate::service::ServiceState;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct ObtainedTokenResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.ey")]
    pub access_token: String,
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.ey")]
    pub refresh_token: String,
    #[schema(example = "1924828424929")]
    pub expires_at: i64,
}

#[axum::debug_handler]
pub async fn request_token_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    CadenceJsonExtractor(payload): CadenceJsonExtractor<ObtainTokenRequest>,
) -> Result<impl IntoResponse, APIResponseError> {
    payload
        .validate()
        .map_err(|details| invalid_input("body", details))?;

    let account = match state
        .services
        .account_service
        .get_from_email_address(&payload.email)
        .await
    {
        Ok(acc) => acc.ok_or_else(|| not_found_entity("account"))?,
        Err(_) => return Err(failed_to_x_account("retrieve")),
    };

    trace!("Account password hash {}", account.password.clone());

    match check_password(&payload.password, &account.password) {
        Ok(true) => {}
        Ok(false) => return Err(invalid_password()),
        Err(_) => return Err(error_hashing_password()),
    }

    let token_service = state.internal.get_token_service();

    let exp = now_millis() + 7 * 24 * 60 * 60 * 1000;
    let access_token = token_service
        .issue(&Claims {
            sub: account.id,
            aud: state.internal.env.get_service_name(),
            exp,
            scope: vec![Scope::Read, Scope::Write],
            token_type: TokenType::Access,
            service: state.internal.env.get_service_metadata(),
        })
        .map_err(|auth_error| error_issueing_token(auth_error))?;

    let refresh_token = token_service
        .issue(&Claims {
            sub: account.id,
            aud: state.internal.env.get_service_name(),
            exp: now_millis() + 2 * 7 * 24 * 60 * 60 * 1000,
            scope: vec![Scope::Read, Scope::Write],
            token_type: TokenType::Refresh,
            service: state.internal.env.get_service_metadata(),
        })
        .map_err(|auth_error| error_issueing_token(auth_error))?;

    Ok(APIResponse::<ObtainedTokenResponse>::success(
        ObtainedTokenResponse {
            access_token,
            refresh_token,
            expires_at: exp,
        },
        cadence_common::api::response::APIResponseObjectType::Account,
    ))
}
