use std::sync::Arc;

use crate::{middlewares::auth::Authenticated, service::ServiceState};
use axum::{extract::State, response::IntoResponse};
use cadence_common::api::{
    error::APIResponseError,
    response::{APIResponse, APIResponseObjectType},
    state::ApplicationState,
};
use serde_json::{Value, json};

#[axum::debug_handler]
pub async fn validate_token_controller(
    State(_): State<Arc<ApplicationState<ServiceState>>>,
    Authenticated(claims): Authenticated,
) -> Result<impl IntoResponse, APIResponseError> {
    Ok(APIResponse::<Value>::success(
        json!(claims),
        APIResponseObjectType::Auth,
    ))
}
