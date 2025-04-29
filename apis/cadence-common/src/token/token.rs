use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
    errors::ErrorKind,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use utoipa::ToSchema;

use crate::{
    api::service::service::APIServiceMetadata,
    error::AuthError,
    types::{ID, Timestamp},
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Write,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenService {
    pub algorithm: Algorithm,
    pub key: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: ID,
    pub aud: String,
    pub exp: Timestamp,
    pub token_type: TokenType,
    pub scope: Vec<Scope>,
    pub service: APIServiceMetadata,
}

impl TokenService {
    pub fn issue(&self, claims: &Claims) -> Result<String, AuthError> {
        let header = Header::new(self.algorithm);

        let encoding_key = EncodingKey::from_secret(self.key.as_bytes());

        Ok(encode(&header, claims, &encoding_key)
            .map_err(|e| AuthError::InternalServerError(e.to_string()))?)
    }

    pub fn validate(
        &self,
        token: &str,
        expected_aud: &str,
    ) -> Result<TokenData<Claims>, AuthError> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_required_spec_claims(&["sub", "exp", "scope"]);
        validation.set_audience(&[expected_aud]);

        Ok(decode::<Claims>(
            &token,
            &DecodingKey::from_secret(self.key.as_bytes()),
            &validation,
        )
        .map_err(|err| {
            debug!("Token validation error: {:?}", err);
            match *err.kind() {
                ErrorKind::InvalidIssuer => AuthError::InvalidIssuer("invalid issuer".to_owned()),
                ErrorKind::InvalidAudience => {
                    AuthError::InvalidAudience("invalid audience".to_owned())
                }
                ErrorKind::InvalidSubject => {
                    AuthError::InvalidSubject("invalid subject".to_owned())
                }
                ErrorKind::InvalidSignature => {
                    AuthError::InvalidSignature("invalid signature".to_owned())
                }
                ErrorKind::ExpiredSignature => AuthError::ExpiredToken("expired token".to_owned()),
                _ => AuthError::InvalidToken("invalid token".to_owned()),
            }
        })?)
    }
}
