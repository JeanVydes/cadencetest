use sea_orm::entity::prelude::*;
use serde::Serialize;

use crate::types::{ID, Timestamp};

/// # Account
///
/// The `account` table stores information about user accounts.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "account")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    /// # Name
    ///
    /// The `name` field stores the name of the account holder.
    /// It is an optional field, meaning it can be `NULL`.
    #[sea_orm(column_type = "Text", column_name = "name", nullable)]
    pub name: Option<String>,

    /// # Country
    ///
    /// The `country` field stores the country code of the account.
    /// It is a 2-character string that follows the ISO 3166-1 alpha-2 standard.
    /// For example, "US" for the United States, "CA" for Canada, etc.
    /// This field is indexed for faster lookups.
    #[sea_orm(
        column_type = "Uuid",
        column_name = "country_code_id"
    )]
    pub country_code_id: ID,

    /// # Password
    ///
    /// The `password` field stores the hashed password of the account holder.
    pub password: String,

    #[sea_orm(column_type = "BigInteger", column_name = "deleted_at", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AccountFlag,
    ExternalIdentity,
    Country,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AccountFlag => Entity::has_many(crate::entities::account::account_flag::Entity)
                .from(Column::Id)
                .to(crate::entities::account::account_flag::Column::AccountId)
                .into(),
            Self::ExternalIdentity => 
                Entity::has_many(crate::entities::account::external_identity::Entity)
                    .from(Column::Id)
                    .to(crate::entities::account::external_identity::Column::AccountId)
                    .into(),
            Self::Country => Entity::belongs_to(crate::entities::country::Entity)
                .from(Column::CountryCodeId)
                .to(crate::entities::country::Column::Id)
                .into(),
        }
    }
}

impl Related<crate::entities::account::account_flag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountFlag.def()
    }
}

impl Related<crate::entities::account::external_identity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExternalIdentity.def()
    }
}

impl Related<crate::entities::country::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Country.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
