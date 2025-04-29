use std::sync::Arc;

use crate::{middlewares::auth::Authenticated, responses::invalid_token, service::ServiceState};
use axum::{extract::State, response::IntoResponse};
use cadence_common::{api::service::service::EnviromentCommon, error::AuthError, time::now_millis, token::token::{Claims, TokenType}};
use cadence_common::api::{
    error::APIResponseError,
    response::{APIResponse, APIResponseObjectType},
    state::ApplicationState,
};
use serde_json::{Value, json};

use super::request_token::ObtainedTokenResponse;

#[axum::debug_handler]
pub async fn validate_token_controller(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    Authenticated(claims): Authenticated,
) -> Result<impl IntoResponse, APIResponseError> {
    if claims.token_type != TokenType::Refresh {
        return Err(invalid_token(AuthError::InvalidToken(
            "Token is not a refresh token".to_string(),
        )));
    }

    let exp = now_millis() + 7 * 24 * 60 * 60 * 1000;
    let access_token = state
        .internal
        .get_token_service()
        .issue(&Claims {
            sub: claims.sub,
            token_type: TokenType::Access,
            scope: claims.scope.clone(),
            aud: state.internal.env.get_service_name(),
            exp,
            service: state.internal.env.get_service_metadata(),
        })
        .map_err(|auth_error| invalid_token(auth_error))?;

    let refresh_token = state
        .internal
        .get_token_service()
        .issue(&Claims {
            sub: claims.sub,
            token_type: TokenType::Refresh,
            scope: claims.scope,
            aud: state.internal.env.get_service_name(),
            exp: now_millis() + 2 * 7 * 24 * 60 * 60 * 1000,
            service: state.internal.env.get_service_metadata(),
        })
        .map_err(|auth_error| invalid_token(auth_error))?;

    Ok(APIResponse::<Value>::success(
        json!(ObtainedTokenResponse {
            access_token,
            refresh_token,
            expires_at: exp,
        }),
        APIResponseObjectType::Auth,
    ))
}
