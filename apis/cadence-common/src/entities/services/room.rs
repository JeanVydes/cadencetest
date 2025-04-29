use crate::entities::account::repositories::account::AccountRepository;
use crate::entities::room::member::{self, Entity as MemberEntity, Model as MemberModel};
use crate::entities::room::message::{self, MessageType, Model as MessageModel};
use crate::entities::room::repositories::member::{
    CreationSchema as MemberCreationSchema, MemberRepository,
};
use crate::entities::room::repositories::message::{
    CreationSchema as MessageCreationSchema, MessageRepository,
};
use crate::entities::room::repositories::room::{
    CreationSchema as RoomCreationSchema, RoomRepository,
};
use crate::entities::room::repositories::template::{
    CreationSchema as RoomTemplateCreationSchema, RoomTemplateRepository,
};
use crate::entities::room::room::Model as RoomModel;
use crate::entities::room::template::{self, Model as RoomTemplateModel};
use crate::error::DatabaseError;
use crate::repository_traits::BasicApplicationService;
use crate::repository_traits::CrudEntityRepository;
use crate::time::now_millis;
use crate::types::ID;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::prelude::*;
use sea_orm::{Order, TransactionTrait};
use serde::{Deserialize, Serialize};

/// # Room Service
///
/// This struct provides a service for managing rooms.
#[derive(Clone, Debug)]
pub struct RoomService {
    pub db: sea_orm::DatabaseConnection,
    pub account_repository: AccountRepository,
    pub room_repository: RoomRepository,
    pub member_repository: MemberRepository,
    pub room_template_repository: RoomTemplateRepository,
    pub message_repository: MessageRepository,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoomServiceCreationSchema {
    pub room: RoomCreationSchema,
    pub author: MemberCreationSchema,
}

/// # Account Service
///
/// This service is responsible for managing accounts and their associations.
impl RoomService {
    pub async fn create_room_and_author_membership(
        &self,
        mut schema: RoomServiceCreationSchema,
    ) -> Result<(RoomModel, Vec<MemberModel>), DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if !self
            .account_repository
            .exists_tx(schema.author.account_id, &txn)
            .await
            .map_err(|_| DatabaseError::QueryFailed("account".to_string()))?.0
        {
            return Err(DatabaseError::RecordNotFound("account".to_string()));
        }

        let room = self
            .room_repository
            .create_tx(&schema.room, &txn)
            .await
            .map_err(|_| DatabaseError::InsertionError("room".to_string()))?;

        schema.author.room_id = room.id;

        let member = self
            .member_repository
            .create_tx(&schema.author, &txn)
            .await
            .map_err(|_| DatabaseError::InsertionError("member".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok((room, vec![member]))
    }

    pub async fn delete_room(
        &self,
        room_id: ID,
        trigger_account_id: Option<ID>,
    ) -> Result<RoomModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if let Some(account_id) = trigger_account_id {
            if !self.has_room_ownership(room_id, account_id).await? {
                return Err(DatabaseError::ConstraintViolation(
                    "trigger account_id is not owner".to_string(),
                ));
            }
        }

        let room = self
            .room_repository
            .delete_tx(room_id, &txn)
            .await
            .map_err(|_| DatabaseError::DeletionError("room".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(room)
    }

    pub async fn get_member_by_account_id(
        &self,
        room_id: ID,
        account_id: ID,
    ) -> Result<Option<MemberModel>, DatabaseError> {
        let member = MemberEntity::find()
            .filter(member::Column::RoomId.eq(room_id))
            .filter(member::Column::AccountId.eq(account_id))
            .one(self.db())
            .await
            .map_err(|_| DatabaseError::QueryFailed("membership".to_string()))?;

        if let Some(ref member) = member {
            if member.deleted_at.is_some() {
                return Ok(None);
            }
        }

        Ok(member)
    }

    pub async fn add_member(
        &self,
        room_id: ID,
        trigger_account_id: Option<ID>,
        account_id: ID,
        anonymize: bool,
    ) -> Result<MemberModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if !self
            .room_repository
            .exists_tx(room_id, &txn)
            .await
            .map_err(|_| DatabaseError::QueryFailed("room".to_string()))?.0
        {
            return Err(DatabaseError::RecordNotFound("room".to_string()));
        }

        if let Some(account_id) = trigger_account_id {
            if !self.has_room_ownership(room_id, account_id).await? {
                return Err(DatabaseError::ConstraintViolation(
                    "trigger account_id is not owner".to_string(),
                ));
            }
        }

        if !self
            .account_repository
            .exists_tx(account_id, &txn)
            .await
            .map_err(|_| DatabaseError::QueryFailed("account".to_string()))?.0
        {
            return Err(DatabaseError::RecordNotFound("account".to_string()));
        }

        let member = self
            .member_repository
            .create_tx(
                &MemberCreationSchema {
                    room_id,
                    account_id,
                    is_owner: false,
                    anonymize,
                },
                &txn,
            )
            .await
            .map_err(|_| DatabaseError::InsertionError("member".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(member)
    }

    // soft delete (repository already does this)
    pub async fn remove_member(
        &self,
        room_id: ID,
        trigger_account_id: Option<ID>,
        account_id: ID,
    ) -> Result<MemberModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if let Some(account_id) = trigger_account_id {
            if !self.has_room_ownership(room_id, account_id).await? {
                return Err(DatabaseError::ConstraintViolation(
                    "trigger account_id is not owner".to_string(),
                ));
            }
        }

        let target_membership = self
            .get_member_by_account_id(room_id, account_id)
            .await
            .map_err(|_| DatabaseError::QueryFailed("membership".to_string()))?
            .ok_or_else(|| DatabaseError::RecordNotFound("membership".to_string()))?;

        if target_membership.is_owner {
            return Err(DatabaseError::ConstraintViolation(
                "cannot remove owner".to_string(),
            ));
        }

        let member = self
            .member_repository
            .delete_tx(target_membership.id, &txn)
            .await
            .map_err(|_| DatabaseError::DeletionError("member".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(member)
    }

    pub async fn save_template(
        &self,
        schema: RoomTemplateCreationSchema,
    ) -> Result<RoomTemplateModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if let Some(source_room_id) = schema.source_room_id {
            if !self
                .room_repository
                .exists_tx(source_room_id, &txn)
                .await
                .map_err(|_| DatabaseError::QueryFailed("room".to_string()))?.0
            {
                return Err(DatabaseError::RecordNotFound("room".to_string()));
            }
        }

        let room_template = self
            .room_template_repository
            .create_tx(&schema, &txn)
            .await
            .map_err(|_| DatabaseError::InsertionError("room".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(room_template)
    }

    pub async fn delete_template(
        &self,
        template_id: ID,
        trigger_account_id: Option<ID>,
    ) -> Result<RoomTemplateModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if let Some(account_id) = trigger_account_id {
            if !self.has_template_ownership(template_id, account_id).await? {
                return Err(DatabaseError::ConstraintViolation(
                    "trigger account_id is not owner".to_string(),
                ));
            }
        }

        let room_template = self
            .room_template_repository
            .delete_tx(template_id, &txn)
            .await
            .map_err(|_| DatabaseError::DeletionError("room".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(room_template)
    }

    pub async fn add_message(
        &self,
        mut schema: MessageCreationSchema,
    ) -> Result<MessageModel, DatabaseError> {
        match (&schema.message_type, &schema.system) {
            (MessageType::Default, true) => {
                return Err(DatabaseError::ConstraintViolation(
                    "system message type must be system".to_string(),
                ));
            }
            (MessageType::RecipientAdded, false) => {
                return Err(DatabaseError::ConstraintViolation(
                    "recipient added message type must be system".to_string(),
                ));
            }
            (MessageType::RecipientRemoved, false) => {
                return Err(DatabaseError::ConstraintViolation(
                    "recipient removed message type must be system".to_string(),
                ));
            }
            (MessageType::Default, false) => {}
            (MessageType::RecipientAdded, true) => {}
            (MessageType::RecipientRemoved, true) => {}
        }

        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        if !self
            .room_repository
            .exists_tx(schema.room_id, &txn)
            .await
            .map_err(|_| DatabaseError::QueryFailed("room".to_string()))?.0
        {
            return Err(DatabaseError::RecordNotFound("room".to_string()));
        }

        // if there is a author, check that the author is a member of the room
        if let Some(ref author_id) = schema.member_id {
            let account_membership = self
                .member_repository
                .get_by_id(author_id.clone())
                .await
                .map_err(|_| DatabaseError::QueryFailed("membership".to_string()))?
                .ok_or_else(|| DatabaseError::RecordNotFound("membership".to_string()))?;

            if account_membership.room_id != schema.room_id {
                return Err(DatabaseError::ConstraintViolation(
                    "member does not belong to room".to_string(),
                ));
            }

            if account_membership.deleted_at.is_some() {
                return Err(DatabaseError::ConstraintViolation(
                    "member is deleted".to_string(),
                ));
            }

            schema.member_id = Some(account_membership.id);
        }

        // if there is a reply_to, check that the message exists
        if let Some(ref reply_to) = schema.reply_to {
            let message = self
                .message_repository
                .get_by_id(reply_to.clone())
                .await
                .map_err(|_| DatabaseError::QueryFailed("message".to_string()))?
                .ok_or_else(|| DatabaseError::RecordNotFound("message".to_string()))?;

            if message.room_id != schema.room_id {
                return Err(DatabaseError::ConstraintViolation(
                    "message does not belong to room".to_string(),
                ));
            }
            if message.deleted_at.is_some() {
                return Err(DatabaseError::ConstraintViolation(
                    "message is deleted".to_string(),
                ));
            }
            if message.is_hidden {
                return Err(DatabaseError::ConstraintViolation(
                    "message is hidden".to_string(),
                ));
            }
        }

        let message = self
            .message_repository
            .create_tx(&schema, &txn)
            .await
            .map_err(|_| DatabaseError::InsertionError("message".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(message)
    }

    pub async fn remove_message(
        &self,
        room_id: ID,
        message_id: ID,
        trigger_account_id: ID,
    ) -> Result<MessageModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed(
                "Failed to start remove_message transaction".to_string(),
            )
        })?;

        let message_to_delete = self
            .message_repository
            .get_by_id(message_id)
            .await
            .map_err(|e| DatabaseError::QueryFailed(format!("Failed message query: {}", e)))?
            .ok_or_else(|| {
                DatabaseError::RecordNotFound(format!("Message {} not found", message_id))
            })?;

        if message_to_delete.room_id != room_id {
            return Err(DatabaseError::ConstraintViolation(format!(
                "Message {} does not belong to room {}",
                message_id, room_id
            )));
        }

        if message_to_delete.deleted_at.is_some() {
            return Err(DatabaseError::ConstraintViolation(format!(
                "Message {} is already deleted",
                message_id
            )));
        }

        if message_to_delete.member_id.is_none() {
            return Err(DatabaseError::ConstraintViolation(format!(
                "Message {} does not have a member_id",
                message_id
            )));
        }

        let member_id = message_to_delete.member_id.unwrap();

        let trigger_membership = self
            .get_member_by_account_id(room_id, trigger_account_id)
            .await?
            .ok_or_else(|| {
                DatabaseError::RecordNotFound(format!(
                    "Trigger account {} not found in room {}",
                    trigger_account_id, room_id
                ))
            })?;

        let author_membership = self
            .member_repository
            .get_by_id(member_id)
            .await
            .map_err(|_| DatabaseError::QueryFailed("membership".to_string()))?
            .ok_or_else(|| {
                DatabaseError::RecordNotFound(format!(
                    "Message author {} not found in room {}",
                    message_id,
                    room_id
                ))
            })?;

        let is_author = author_membership.account_id == trigger_account_id;
        let is_owner = trigger_membership.is_owner;

        if !is_author && !is_owner {
            return Err(DatabaseError::ConstraintViolation(
                "User is not the message author or room owner".to_string(),
            ));
        }

        let deleted_message = self
            .message_repository
            .delete_tx(message_id, &txn)
            .await
            .map_err(|e| {
                DatabaseError::DeletionError(format!(
                    "Failed to delete message {}: {}",
                    message_id, e
                ))
            })?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed(
                "Failed to commit remove_message transaction".to_string(),
            )
        })?;

        Ok(deleted_message)
    }

    pub async fn has_room_ownership(
        &self,
        room_id: ID,
        account_id: ID,
    ) -> Result<bool, DatabaseError> {
        let member = self
            .get_member_by_account_id(room_id, account_id)
            .await
            .map_err(|_| DatabaseError::QueryFailed("membership".to_string()))?;

        if member.is_none() {
            return Ok(false);
        }

        let member = member.unwrap();
        if member.deleted_at.is_some() {
            return Ok(false);
        }

        Ok(member.is_owner)
    }

    pub async fn has_template_ownership(
        &self,
        template_id: ID,
        account_id: ID,
    ) -> Result<bool, DatabaseError> {
        let template = self
            .room_template_repository
            .get_by_id(template_id)
            .await
            .map_err(|_| DatabaseError::QueryFailed("template".to_string()))?;

        if template.is_none() {
            return Ok(false);
        }

        let template = template.unwrap();
        if template.deleted_at.is_some() {
            return Ok(false);
        }

        Ok(template.author_id == Some(account_id))
    }

    pub async fn get_messages(
        &self,
        room_id: ID,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<MessageModel>, DatabaseError> {
        if limit > 100 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be less than 100".to_string(),
            ));
        }
        if offset > 1000 {
            return Err(DatabaseError::ConstraintViolation(
                "offset must be less than 1000".to_string(),
            ));
        }
        if limit == 0 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be greater than 0".to_string(),
            ));
        }

        let messages = message::Entity::find()
            .filter(message::Column::RoomId.eq(room_id))
            .order_by(message::Column::CreatedAt, Order::Desc)
            .limit(limit)
            .offset(offset)
            .all(self.db())
            .await
            .map_err(|_| DatabaseError::QueryFailed("messages".to_string()))?;

        Ok(messages)
    }

    pub async fn get_members(
        &self,
        room_id: ID,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<MemberModel>, DatabaseError> {
        if limit > 100 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be less than 100".to_string(),
            ));
        }
        if offset > 1000 {
            return Err(DatabaseError::ConstraintViolation(
                "offset must be less than 1000".to_string(),
            ));
        }
        if limit == 0 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be greater than 0".to_string(),
            ));
        }

        let members = member::Entity::find()
            .filter(member::Column::RoomId.eq(room_id))
            .order_by(member::Column::CreatedAt, Order::Desc)
            .filter(member::Column::DeletedAt.is_null())
            .filter(member::Column::BannedAt.is_null())
            .limit(limit)
            .offset(offset)
            .all(self.db())
            .await
            .map_err(|_| DatabaseError::QueryFailed("members".to_string()))?;

        Ok(members)
    }

    pub async fn search_templates(
        &self,
        query: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<RoomTemplateModel>, DatabaseError> {
        if limit > 10 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be less than 10".to_string(),
            ));
        }
        if offset > 100 {
            return Err(DatabaseError::ConstraintViolation(
                "offset must be less than 1000".to_string(),
            ));
        }
        if limit == 0 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be greater than 0".to_string(),
            ));
        }

        let templates = template::Entity::find()
            .filter(template::Column::DeletedAt.is_null())
            .filter(template::Column::Name.contains(query))
            .order_by(template::Column::CreatedAt, Order::Desc)
            .limit(limit)
            .offset(offset)
            .all(self.db())
            .await
            .map_err(|_| DatabaseError::QueryFailed("templates".to_string()))?;

        Ok(templates)
    }

    pub async fn search_messages(
        &self,
        room_id: ID,
        query: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<MessageModel>, DatabaseError> {
        if limit > 10 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be less than 100".to_string(),
            ));
        }
        if offset > 100 {
            return Err(DatabaseError::ConstraintViolation(
                "offset must be less than 1000".to_string(),
            ));
        }
        if limit == 0 {
            return Err(DatabaseError::ConstraintViolation(
                "limit must be greater than 0".to_string(),
            ));
        }

        let messages = message::Entity::find()
            .filter(message::Column::IsHidden.eq(false))
            .filter(message::Column::DeletedAt.is_null())
            .filter(message::Column::RoomId.eq(room_id))
            .filter(message::Column::Content.contains(query))
            .order_by(message::Column::CreatedAt, Order::Desc)
            .limit(limit)
            .offset(offset)
            .all(self.db())
            .await
            .map_err(|_| DatabaseError::QueryFailed("messages".to_string()))?;

        Ok(messages)
    }

    pub async fn toggle_pin_message(
        &self,
        room_id: ID,
        message_id: ID,
        trigger_account_id: ID,
    ) -> Result<MessageModel, DatabaseError> {
        let txn = self.db().begin().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to start transaction".to_string())
        })?;

        let mut message_to_pin = self
            .message_repository
            .get_by_id(message_id)
            .await
            .map_err(|_| DatabaseError::QueryFailed("message".to_string()))?
            .ok_or_else(|| {
                DatabaseError::RecordNotFound(format!("Message {} not found", message_id))
            })?;

        if message_to_pin.room_id != room_id {
            return Err(DatabaseError::ConstraintViolation(format!(
                "Message {} does not belong to room {}",
                message_id, room_id
            )));
        }

        if message_to_pin.deleted_at.is_some() {
            return Err(DatabaseError::ConstraintViolation(format!(
                "Message {} is already deleted",
                message_id
            )));
        }

        let trigger_membership = self
            .get_member_by_account_id(room_id, trigger_account_id)
            .await?
            .ok_or_else(|| {
                DatabaseError::RecordNotFound(format!(
                    "Trigger account {} not found in room {}",
                    trigger_account_id, room_id
                ))
            })?;

        let is_owner = trigger_membership.is_owner;

        if !is_owner {
            return Err(DatabaseError::ConstraintViolation(
                "User is not the room owner".to_string(),
            ));
        }

        if message_to_pin.pinned_at.is_some() {
            message_to_pin.pinned_at = None;
        } else {
            message_to_pin.pinned_at = Some(now_millis());
        }

        let pinned_message = self
            .message_repository
            .update_tx(message_id, message_to_pin.into(), &txn)
            .await
            .map_err(|_| DatabaseError::UpdateError("message".to_string()))?;

        txn.commit().await.map_err(|_| {
            DatabaseError::TransactionFailed("Failed to commit transaction".to_string())
        })?;

        Ok(pinned_message)
    }
}

impl BasicApplicationService for RoomService {
    type DatabaseConnection = sea_orm::DatabaseConnection;

    fn new(db: sea_orm::DatabaseConnection) -> Self {
        RoomService {
            db: db.clone(),
            account_repository: AccountRepository::new(db.clone()),
            member_repository: MemberRepository::new(db.clone()),
            room_repository: RoomRepository::new(db.clone()),
            room_template_repository: RoomTemplateRepository::new(db.clone()),
            message_repository: MessageRepository::new(db.clone()),
        }
    }

    fn db(&self) -> &Self::DatabaseConnection {
        &self.db
    }
}
