use cadence_common::{entities::account::account::Model, types::Timestamp};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct CensoredAccountResponse {
    #[schema(example = json!(uuid::Uuid::new_v4()))]
    pub id: String,
    #[schema(example = "John Doe", nullable = true)]
    pub name: Option<String>,
    #[schema(example = "US")]
    pub country_code: String,
    #[schema(value_type = i64, example = 1)]
    pub created_at: Timestamp,
    #[schema(value_type = i64, example = 1)]
    pub updated_at: Timestamp,
}


impl From<Model> for CensoredAccountResponse {
    fn from(account_model: Model) -> Self {
        CensoredAccountResponse {
            id: account_model.id.to_string(),
            name: account_model.name,
            country_code: account_model.country_code_id.to_string(),
            created_at: account_model.created_at,
            updated_at: account_model.updated_at,
        }
    }
}