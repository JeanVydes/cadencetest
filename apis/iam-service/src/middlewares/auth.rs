use std::sync::Arc;

use crate::{responses::invalid_token, service::ServiceState};
use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use cadence_common::api::service::service::EnviromentCommon;
use cadence_common::{
    api::{error::APIResponseError, state::ApplicationState},
    error::AuthError,
    token::token::Claims,
};

#[derive(Clone, Debug)]
pub struct Authenticated(pub Claims);

pub async fn require_authentication(
    State(state): State<Arc<ApplicationState<ServiceState>>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, APIResponseError> {
    let token_str = auth_header
        .ok_or_else(|| {
            invalid_token(AuthError::InvalidToken(
                "Authorization header missing".to_string(),
            ))
        })?
        .token() // Get the token part from the Bearer header
        .to_string();

    let token_data = state
        .internal
        .get_token_service()
        .validate(&token_str, &state.internal.env.get_service_name())
        .map_err(|auth_error| invalid_token(auth_error))?;

    request
        .extensions_mut()
        .insert(Authenticated(token_data.claims));

    Ok(next.run(request).await)
}

impl<S> FromRequestParts<Arc<ApplicationState<S>>> for Authenticated
where
    S: Send + Sync + 'static,
    Arc<ApplicationState<S>>: Send + Sync + 'static,
{
    type Rejection = APIResponseError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &Arc<ApplicationState<S>>, // State isn't strictly needed here, but required by trait
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Authenticated>()
            .cloned()
            .ok_or_else(|| {
                invalid_token(AuthError::InvalidToken(
                    "Authenticated context extension missing".to_string(),
                ))
            })
            .map(|authenticated| authenticated)
    }
}
