use crate::entities::account::account::Model as AccountModel;
use crate::entities::account::email::Model as EmailModel;
use crate::entities::account::external_identity::{Model as ExternalIdentityModel, Provider};
use crate::entities::account::repositories::account::{
    AccountRepository, CreationSchema as AccountCreationSchema,
};
use crate::entities::account::repositories::email::{
    CreationSchema as EmailCreationSchema, EmailRepository,
};
use crate::entities::account::{self, account_email, account_flag, external_identity, flag};
use crate::error::DatabaseError;
use crate::repository_traits::BasicApplicationService;
use crate::repository_traits::CrudEntityRepository;
use crate::time::now_millis;
use crate::types::ID;
use sea_orm::ActiveValue::Set;
use sea_orm::TransactionTrait;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::trace;

/// # Account Service
///
/// This struct provides a service for managing accounts and their associated emails.
#[derive(Clone, Debug)]
pub struct AccountService {
    pub db: sea_orm::DatabaseConnection,
    pub account_repository: AccountRepository,
    pub email_repository: EmailRepository,
}

/// # Account Service Creation Schema
///
/// This struct provides info to create an account with emails.
/// It contains the account creation schema and a vector of email creation schemas.
/// The `AccountCreationSchema` is used to create the account, and the `EmailCreationSchema` is used
/// to create the emails.
/// The `emails` vector can contain multiple email creation schemas, allowing the creation of
/// multiple emails associated with the account.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountServiceCreationSchema {
    pub account: AccountCreationSchema,
    pub emails: Vec<EmailCreationSchema>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountServiceUpdateSchema {
    pub name: Option<String>,
    pub country_code_id: Option<Uuid>,
    pub password: Option<String>,
}

/// # Account Service 3rd Party Creation Schema
///
/// This struct provide info to create an account with a 3rd party provider.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountService3rdPartyCreationSchema {
    pub provider: Provider,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub encrypted_refresh_token: Option<String>,
}

/// # Account Service
///
/// This service is responsible for managing accounts and their associations.
impl AccountService {
    /// ## Create an account with emails
    ///
    /// This function creates an account with emails. It first creates the account using the `account_repository`.
    /// Then, for each email in the `emails` vector, it creates the email using the `email_repository` and
    /// establishes a relationship between the account and the email. Finally, it commits the transaction and
    /// returns the account and the emails.
    pub async fn create_with_emails(
        &self,
        schema: AccountServiceCreationSchema,
    ) -> Result<(AccountModel, Vec<EmailModel>), DatabaseError> {
        let txn = self.db().begin().await.map_err(|e| {
            trace!("Error starting transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        let account = self
            .account_repository
            .create_tx(&schema.account, &txn)
            .await
            .map_err(|e| {
                trace!("Error creating account: {:?}", e);
                DatabaseError::InsertionError("account".to_string())
            })?;

        let mut emails_models = Vec::new();
        for email in schema.emails {
            let email_model = self
                .email_repository
                .create_tx(&email, &txn)
                .await
                .map_err(|e| {
                    trace!("Error creating email: {:?}", e);
                    DatabaseError::InsertionError("email".to_string())
                })?;

            let account_email = account_email::ActiveModel {
                created_at: Set(now_millis()),
                updated_at: Set(now_millis()),
                account_id: Set(account.id),
                email_id: Set(email_model.id),
                ..Default::default()
            };

            account_email.insert(&txn).await.map_err(|e| {
                trace!("Error creating account email relationship: {:?}", e);
                DatabaseError::InsertionError("account_email".to_string())
            })?;

            emails_models.push(email_model.to_owned());
        }

        txn.commit().await.map_err(|e| {
            trace!("Error committing transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok((account, emails_models))
    }

    /// ## Create an account with a 3rd party provider
    ///
    /// This function creates an account with a 3rd party provider. It first creates the account using the
    /// `account_repository`. If an email is provided, it creates the email using the `email_repository` and
    /// establishes a relationship between the account and the email. Finally, it creates an external identity
    /// using the `external_identity` model and commits the transaction.
    pub async fn create_with_provider(
        &self,
        account_schema: AccountServiceCreationSchema,
        provider_schema: AccountService3rdPartyCreationSchema,
    ) -> Result<(AccountModel, ExternalIdentityModel, Vec<EmailModel>), DatabaseError> {
        let txn = self.db().begin().await.map_err(|e| {
            trace!("Error starting transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        let account = self
            .account_repository
            .create_tx(&account_schema.account, &txn)
            .await
            .map_err(|e| {
                trace!("Error creating account: {:?}", e);
                DatabaseError::InsertionError("account".to_string())
            })?;

        let mut possible_emails = Vec::new();
        if let Some(email) = provider_schema.email {
            let email_model = self
                .email_repository
                .create_tx(
                    &EmailCreationSchema {
                        email,
                        primary: true,
                        verification_code: None,
                    },
                    &txn,
                )
                .await
                .map_err(|e| {
                    trace!("Error creating email: {:?}", e);
                    DatabaseError::InsertionError("email".to_string())
                })?;

            let email_id = email_model.id.clone();
            possible_emails.push(email_model);

            // the relationship between account and email
            let account_email = account_email::ActiveModel {
                created_at: Set(now_millis()),
                updated_at: Set(now_millis()),
                account_id: Set(account.id),
                email_id: Set(email_id),
                ..Default::default()
            };

            account_email.insert(&txn).await.map_err(|e| {
                trace!("Error creating account email relationship: {:?}", e);
                DatabaseError::InsertionError("account_email".to_string())
            })?;
        }

        // create the external identity
        let external_identity = external_identity::ActiveModel {
            account_id: Set(account.id),
            provider: Set(provider_schema.provider),
            provider_user_id: Set(provider_schema.provider_user_id),
            name: Set(provider_schema.name),
            avatar_url: Set(provider_schema.avatar_url),
            encrypted_refresh_token: Set(provider_schema.encrypted_refresh_token),
            created_at: Set(now_millis()),
            updated_at: Set(now_millis()),
            ..Default::default()
        };

        let external_identity_model = external_identity.insert(&txn).await.map_err(|e| {
            trace!("Error creating external identity: {:?}", e);
            DatabaseError::InsertionError("external_identity".to_string())
        })?;

        txn.commit().await.map_err(|e| {
            trace!("Error committing transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok((account, external_identity_model, possible_emails))
    }

    /// ## Add flags to an account
    ///
    /// This function adds flags to an account. It first retrieves the account by its ID, then
    /// retrieves the flags by their IDs. It creates a relationship between the account and each flag
    /// in the `account_flag` join table. Finally, it commits the transaction and returns the account and flags.
    pub async fn add_flags(
        &self,
        account_id: ID,
        flags: Vec<ID>,
    ) -> Result<(AccountModel, Vec<flag::Model>), DatabaseError> {
        let account = self
            .account_repository
            .get_by_id(account_id)
            .await
            .map_err(|e| {
                trace!("Error getting account by id: {:?}", e);
                DatabaseError::QueryFailed("Failed to get account by id".to_string())
            })?
            .ok_or(DatabaseError::RecordNotFound(
                "Account not found".to_string(),
            ))?;

        let tx = self.db().begin().await.map_err(|e| {
            trace!("Error starting transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        let flags = flag::Entity::find()
            .filter(flag::Column::Id.is_in(flags))
            .all(&tx)
            .await
            .map_err(|e| {
                trace!("Error getting flags: {:?}", e);
                DatabaseError::QueryFailed("Failed to get flags".to_string())
            })?;

        for flag in flags.clone() {
            let relationship = account_flag::ActiveModel {
                created_at: Set(now_millis()),
                updated_at: Set(now_millis()),
                account_id: Set(account.id),
                flag_id: Set(flag.id),
                ..Default::default()
            };

            relationship.insert(&tx).await.map_err(|e| {
                trace!("Error creating account flag relationship: {:?}", e);
                DatabaseError::InsertionError("account_flag".to_string())
            })?;
        }

        tx.commit().await.map_err(|e| {
            trace!("Error committing transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok((account, flags))
    }

    /// ## Remove Flag
    ///
    /// Removes a flag from an account by deleting the relationship between the account and the flag at `account_flag` join table
    pub async fn remove_flags(
        &self,
        account_id: ID,
        flags: Vec<ID>,
    ) -> Result<(AccountModel, Vec<flag::Model>), DatabaseError> {
        let account = self
            .account_repository
            .get_by_id(account_id)
            .await
            .map_err(|e| {
                trace!("Error getting account by id: {:?}", e);
                DatabaseError::QueryFailed("Failed to get account by id".to_string())
            })?
            .ok_or(DatabaseError::RecordNotFound(
                "Account not found".to_string(),
            ))?;

        let tx = self.db().begin().await.map_err(|e| {
            trace!("Error starting transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        let flags = flag::Entity::find()
            .filter(flag::Column::Id.is_in(flags))
            .all(&tx)
            .await
            .map_err(|e| {
                trace!("Error getting flags: {:?}", e);
                DatabaseError::QueryFailed("Failed to get flags".to_string())
            })?;

        for flag in flags.clone() {
            let relationship = account_flag::ActiveModel {
                created_at: Set(now_millis()),
                updated_at: Set(now_millis()),
                account_id: Set(account.id),
                flag_id: Set(flag.id),
                ..Default::default()
            };

            relationship.delete(&tx).await.map_err(|e| {
                trace!("Error deleting account flag relationship: {:?}", e);
                DatabaseError::DeletionError("account_flag".to_string())
            })?;
        }

        tx.commit().await.map_err(|e| {
            trace!("Error committing transaction: {:?}", e);
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok((account, flags))
    }

    pub async fn get_from_email_address(
        &self,
        email_address: &str,
    ) -> Result<Option<AccountModel>, DatabaseError> {
        let email = self
            .email_repository
            .find_by_email(email_address)
            .await
            .map_err(|e| {
                trace!("Error getting email by email: {:?}", e);
                DatabaseError::QueryFailed("Failed to get email by email".to_string())
            })?;

        if let Some(email) = email {
            // now find the relationship with the email_id to get the account_id
            let (_, account) = account_email::Entity::find()
                .find_also_related(account::account::Entity)
                .filter(account_email::Column::EmailId.eq(email.id))
                .one(self.db())
                .await
                .map_err(|e| {
                    trace!("Error getting account email by email: {:?}", e);
                    DatabaseError::QueryFailed("Failed to get account email by email".to_string())
                })?
                .ok_or(DatabaseError::RecordNotFound(
                    "Account email not found".to_string(),
                ))?;

            if account.is_none() {
                return Ok(None);
            }

            return Ok(account);
        }

        Err(DatabaseError::RecordNotFound("Email not found".to_string()))
    }

    pub async fn update(
        &self,
        id: ID,
        schema: AccountServiceUpdateSchema,
    ) -> Result<AccountModel, DatabaseError> {
        let mut model = self
            .account_repository
            .get_by_id(id)
            .await
            .map_err(|e| {
                trace!("Error getting account by id: {:?}", e);
                DatabaseError::QueryFailed("Failed to get account by id".to_string())
            })?
            .ok_or(DatabaseError::RecordNotFound(
                "Account not found".to_string(),
            ))?;

        model.name = schema.name;
        model.updated_at = now_millis();

        if let Some(country_code) = schema.country_code_id {
            model.country_code_id = country_code;
        }

        if let Some(password) = schema.password {
            model.password = password;
        }

        self.account_repository
            .update(id, model.into())
            .await
            .map_err(|e| {
                trace!("Error updating account: {:?}", e);
                DatabaseError::UpdateError("account".to_string())
            })
    }
}

impl BasicApplicationService for AccountService {
    type DatabaseConnection = sea_orm::DatabaseConnection;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        AccountService {
            db: db.clone(),
            account_repository: AccountRepository::new(db.clone()),
            email_repository: EmailRepository::new(db),
        }
    }

    fn db(&self) -> &Self::DatabaseConnection {
        &self.db
    }
}
