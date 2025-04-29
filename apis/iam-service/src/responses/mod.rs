use cadence_common::{
    api::error::{APIResponseError, APIResponseErrorDetail},
    error::{AuthError, CadenceError, DatabaseError, EntityError, InputError, ServerError},
};

pub fn invalid_input(input_format: &str, details: Vec<APIResponseErrorDetail>) -> APIResponseError {
    return APIResponseError::new(
        CadenceError::Input(InputError::InvalidFormat(input_format.to_string())),
        "Input data validation failed.".to_string(),
        details,
    );
}

pub fn error_hashing_password() -> APIResponseError {
    return APIResponseError::new(
        CadenceError::ServerError(ServerError::InternalError(
            "Password hashing failed".to_string(),
        )),
        "Failed to hash password.".to_string(),
        vec![APIResponseErrorDetail::body(
            "password",
            "Password hashing failed.".to_string(),
        )],
    );
}

pub fn not_found_entity(entity: &str) -> APIResponseError {
    return APIResponseError::new(
        CadenceError::Database(DatabaseError::RecordNotFound(
            format!("{} not found", entity).to_string(),
        )),
        format!("{} not found", entity).to_string(),
        Vec::new(),
    );
}

pub fn failed_to_x_account(action: &str) -> APIResponseError {
    return APIResponseError::new(
        CadenceError::ServerError(ServerError::InternalError(
            format!("Failed to {} account", action).to_string(),
        )),
        format!("Failed to {} account due to an internal error.", action).to_string(),
        Vec::new(),
    );
}

pub fn delegated_account_dont_match() -> APIResponseError {
    return APIResponseError::auth_error(
        AuthError::Unauthorized("The delegated account provided by the token credential doesn't match the provided account id".to_owned()),
        "The delegated account provided by the token credential doesn't match the provided account id".to_owned(),
        vec![APIResponseErrorDetail::header(
            "account_id",
            "The delegated account provided by the token credential doesn't match the provided account id".to_owned(),
        )],
    );
}

pub fn invalid_token(auth_error: AuthError) -> APIResponseError {
    return APIResponseError::auth_error(
        auth_error,
        "Token validation failed".to_string(),
        vec![APIResponseErrorDetail::header(
            "Authorization",
            "Invalid or expired token provided.".to_string(),
        )],
    )
}

pub fn invalid_password() -> APIResponseError {
    return APIResponseError::auth_error(
        AuthError::InvalidCredentials("Invalid password".to_string()),
        "Invalid password".to_string(),
        vec![],
    )
}

pub fn error_issueing_token(auth_error: AuthError) -> APIResponseError {
    return APIResponseError::auth_error(
        auth_error,
        "Token issueing failed".to_string(),
        vec![],
    )
}

pub fn entity_already_exists(entity: &str, field: &str, value: &str) -> APIResponseError {
    let detail_msg = format!("{} with this {} already exists.", entity, field);
    let internal_msg = format!(
        "{} with {} '{}' already exists",
        entity, field, value
    );
    
    return APIResponseError::new(
        CadenceError::Entity(EntityError::AlreadyExists(internal_msg)),
        "Resource conflict".to_string(),
        vec![APIResponseErrorDetail::body(field, detail_msg)],
    );
}