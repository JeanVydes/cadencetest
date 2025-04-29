use crate::types::ID;
use crate::types::Timestamp;
use sea_orm::entity::prelude::*;
use serde::{self, Deserialize, Serialize};

/// # Email
///
/// This is the email entity.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sea_orm(table_name = "email")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    #[sea_orm(column_type = "Text", column_name = "email", unique, indexed)]
    pub email: String,

    #[sea_orm(column_type = "Boolean", column_name = "primary")]
    pub primary: bool,
    #[sea_orm(column_type = "BigInteger", column_name = "verified_at")]
    pub verified_at: Option<Timestamp>,

    /// # Verification code
    ///
    /// This is a code that is used to verify the email address.
    /// Is hashed and stored in the database.
    #[sea_orm(column_type = "Text", column_name = "verification_code", nullable)]
    pub verification_code: Option<String>,

    #[sea_orm(column_type = "BigInteger", column_name = "end_time", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AccountEmail,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AccountEmail => Entity::has_many(crate::entities::account::account_email::Entity)
                .from(Column::Id)
                .to(crate::entities::account::account_email::Column::EmailId)
                .into(),
        }
    }
}

impl Related<crate::entities::account::account_email::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountEmail.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
