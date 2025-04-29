use crate::types::ID;
use crate::types::Timestamp;
use sea_orm::entity::prelude::*;
use serde::{self, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "Uuid",
        column_name = "id"
    )]
    pub id: ID,

    #[sea_orm(column_type = "Text", column_name = "name", nullable)]
    pub name: String,
    #[sea_orm(column_type = "Text", column_name = "description", nullable)]
    pub description: Option<String>,
    /// # Color
    ///
    /// This is the color of the event, used for visual representation in calendar applications.
    /// The color is represented as a hexadecimal string (e.g., "#FF5733").
    #[sea_orm(column_type = "Text", column_name = "color", nullable)]
    pub color: Option<String>,

    #[sea_orm(column_type = "BigInteger", column_name = "end_time", nullable)]
    pub deleted_at: Option<Timestamp>,
    #[sea_orm(column_type = "BigInteger", column_name = "created_at", auto_now_add)]
    pub created_at: Timestamp,
    #[sea_orm(column_type = "BigInteger", column_name = "updated_at", auto_now)]
    pub updated_at: Timestamp,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            _ => todo!("Implement relation definition"),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
