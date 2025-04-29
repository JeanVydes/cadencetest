use axum::extract::rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CadenceError {
    Input(InputError),
    Auth(AuthError),
    Entity(EntityError),
    Database(DatabaseError),
    ServerError(ServerError),
}

/// Detailed input validation errors.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InputError {
    #[schema(example = "email")]
    MissingField(String),
    #[schema(example = "userId format is incorrect")]
    InvalidField(String),
    #[schema(example = "age must be positive")]
    InvalidValue(String),
    #[schema(example = "expected number, got string")]
    InvalidType(String),
    #[schema(example = "date must be yyyy-MM-dd")]
    InvalidFormat(String),
    #[schema(example = "password must be at least 8 characters")]
    InvalidLength(String),
    #[schema(example = "value must be between 1 and 100")]
    InvalidRange(String),
    #[schema(example = "field must match regex XYZ")]
    InvalidPattern(String),
    #[schema(example = "status must be one of [active, inactive]")]
    InvalidEnumValue(String),
}

/// Detailed authentication/authorization errors.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthError {
    #[schema(example = "Authentication required")]
    Unauthorized(String),
    #[schema(example = "OAuth request malformed")]
    InvalidRequest(String),
    #[schema(example = "OAuth provider response invalid")]
    InvalidResponse(String),
    #[schema(example = "Incorrect username or password")]
    InvalidCredentials(String),
    #[schema(example = "JWT expired or invalid")]
    InvalidToken(String),
    #[schema(example = "Token signature mismatch")]
    InvalidSignature(String),
    #[schema(example = "Insufficient scope")]
    InvalidScope(String),
    #[schema(example = "Authorization code expired")]
    InvalidGrant(String),
    #[schema(example = "Invalid OAuth client ID")]
    InvalidClient(String),
    #[schema(example = "Redirect URI mismatch")]
    InvalidRedirectUri(String),
    #[schema(example = "Token audience invalid")]
    InvalidAudience(String),
    #[schema(example = "Token issuer invalid")]
    InvalidIssuer(String),
    #[schema(example = "Token subject invalid")]
    InvalidSubject(String),
    #[schema(example = "Error issuing token")]
    InternalServerError(String),
    #[schema(example = "Token expired")]
    ExpiredToken(String),
    #[schema(example = "Authorization header required")]
    MissingToken(String),
    #[schema(example = "Token mismatch")]
    MismatchToken(String),
}

/// Detailed business logic/entity related errors.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityError {
    #[schema(example = "Goal with ID 123 not found")]
    NotFound(String),
    #[schema(example = "Email address 'test@example.com' already registered")]
    AlreadyExists(String),
    #[schema(example = "Cannot modify a completed order")]
    InvalidState(String),
    #[schema(example = "Cannot transition task from 'Done' to 'Todo'")]
    InvalidTransition(String),
    #[schema(example = "User cannot be linked to this project")]
    InvalidAssociation(String),
    #[schema(example = "Inconsistent relation found")]
    InvalidRelation(String),
    #[schema(example = "Referenced entity does not exist")]
    InvalidReference(String),
    #[schema(example = "Foreign key violation on user_id")]
    InvalidForeignKey(String),
    #[schema(example = "Unique constraint violation on username")]
    InvalidUniqueConstraint(String),
    #[schema(example = "Data integrity violation")]
    InvalidIntegrity(String),
    #[schema(example = "Data type mismatch")]
    InvalidDataType(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseError {
    #[schema(example = "Database connection failed")]
    ConnectionFailed(String),
    #[schema(example = "Query execution failed")]
    QueryFailed(String),
    #[schema(example = "Transaction rollback required")]
    TransactionFailed(String),
    #[schema(example = "Database timeout occurred")]
    Timeout(String),
    #[schema(example = "Database deadlock detected")]
    Deadlock(String),
    #[schema(example = "Database constraint violation")]
    ConstraintViolation(String),
    #[schema(example = "Database schema mismatch")]
    InsertionError(String),
    #[schema(example = "Database schema mismatch")]
    UpdateError(String),
    #[schema(example = "Database schema mismatch")]
    DeletionError(String),
    #[schema(example = "Database schema mismatch")]
    RetrievalError(String),
    #[schema(example = "Database schema mismatch")]
    RecordNotFound(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServerError {
    #[schema(example = "Internal server error occurred")]
    InternalError(String),
    #[schema(example = "Service unavailable")]
    ServiceUnavailable(String),
    #[schema(example = "Gateway timeout occurred")]
    GatewayTimeout(String),
    #[schema(example = "Bad request format")]
    BadRequest(String),
    #[schema(example = "Unsupported media type")]
    EnviromentParseError(String),
}

#[derive(Debug)]
pub enum AxumError {
    JsonExtractorRejection(JsonRejection),
    QueryExtractorRejection(QueryRejection),
    PathExtractorRejection(PathRejection),
    FormExtractorRejection(FormRejection),
}
