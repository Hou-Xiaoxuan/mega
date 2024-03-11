//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

use crate::db_enums::StorageType;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "raw_blob")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(unique)]
    pub sha1: String,
    #[sea_orm(column_type = "Text")]
    pub content: Option<String>,
    pub file_type: Option<String>,
    pub storage_type: StorageType,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))", nullable)]
    pub data: Option<Vec<u8>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub local_path: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub remote_url: Option<String>,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}