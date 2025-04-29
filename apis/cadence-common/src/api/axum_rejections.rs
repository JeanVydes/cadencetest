use axum::extract::rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::{CadenceError, InputError};

use super::error::APIResponseError;

#[derive(axum::extract::FromRequest)]
#[from_request(via(axum::extract::Json), rejection(APIResponseError))]
pub struct CadenceJsonExtractor<T>(pub T);

#[derive(axum::extract::FromRequest)]
#[from_request(via(axum::extract::Query), rejection(APIResponseError))]
pub struct CadenceQueryExtractor<T>(pub T);

#[derive(axum::extract::FromRequest)]
#[from_request(via(axum::extract::Form), rejection(APIResponseError))]
pub struct CadenceFormExtractor<T>(pub T);

#[derive(axum::extract::FromRequest)]
#[from_request(via(axum::extract::Path), rejection(APIResponseError))]
pub struct CadencePathExtractor<T>(pub T);

impl<T: Serialize> IntoResponse for CadenceJsonExtractor<T> {
    fn into_response(self) -> Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl<T: Serialize> IntoResponse for CadenceQueryExtractor<T> {
    fn into_response(self) -> Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl<T: Serialize> IntoResponse for CadenceFormExtractor<T> {
    fn into_response(self) -> Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl<T: Serialize> IntoResponse for CadencePathExtractor<T> {
    fn into_response(self) -> Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl From<JsonRejection> for APIResponseError {
    fn from(rejection: JsonRejection) -> Self {
        Self {
            error: CadenceError::Input(InputError::InvalidFormat("invalid body".to_owned())),
            message: rejection.body_text(),
            details: vec![],
        }
    }
}

impl From<QueryRejection> for APIResponseError {
    fn from(rejection: QueryRejection) -> Self {
        Self {
            error: CadenceError::Input(InputError::InvalidFormat("invalid query".to_owned())),
            message: rejection.body_text(),
            details: vec![],
        }
    }
}

impl From<FormRejection> for APIResponseError {
    fn from(rejection: FormRejection) -> Self {
        Self {
            error: CadenceError::Input(InputError::InvalidFormat("invalid form".to_owned())),
            message: rejection.body_text(),
            details: vec![],
        }
    }
}

impl From<PathRejection> for APIResponseError {
    fn from(rejection: PathRejection) -> Self {
        Self {
            error: CadenceError::Input(InputError::InvalidFormat("invalid path".to_owned())),
            message: rejection.body_text(),
            details: vec![],
        }
    }
}
