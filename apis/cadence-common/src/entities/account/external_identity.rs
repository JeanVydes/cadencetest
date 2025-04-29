use crate::types::{Timestamp, ID};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// # External Identity Provider
/// 
/// This enum represents the different external identity providers that can be used.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "snake_case"
)]
pub enum Provider {
    #[sea_orm(string_value = "google")]
    Google,
    #[sea_orm(string_value = "apple")]
    Apple,
    #[sea_orm(string_value = "microsoft")]
    Microsoft,
    #[sea_orm(string_value = "github")]
    Github,
    #[sea_orm(string_value = "facebook")]
    Facebook,
}

/// # External Identity
/// 
/// This entity represents an external identity linked to an account.
/// It is used to store information about the external identity provider,
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "external_identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Uuid", column_name = "id")]
    pub id: ID,

    #[sea_orm(column_type = "Uuid", column_name = "account_id", indexed)]
    pub account_id: ID,

    #[sea_orm(column_type = "Text", indexed)]
    pub provider: Provider,

    #[sea_orm(column_type = "Text", indexed)]
    pub provider_user_id: String,

    #[sea_orm(column_type = "Text", nullable, indexed)]
    pub email: Option<String>,

    #[sea_orm(column_type = "Text", nullable)]
    pub name: Option<String>,

    #[sea_orm(column_type = "Text", nullable)]
    pub avatar_url: Option<String>,

    #[sea_orm(column_type = "Text", nullable)]
    pub encrypted_refresh_token: Option<String>,

    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Account,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Account => Entity::belongs_to(super::account::Entity)
                .from(Column::AccountId)
                .to(super::account::Column::Id)
                .into(),
        }
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
