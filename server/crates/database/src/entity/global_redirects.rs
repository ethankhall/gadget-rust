//! SeaORM Entity. Generated by sea-orm-codegen 0.6.0

use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "global_redirects"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub global_redirects_id: i32,
    pub public_ref: String,
    pub alias: String,
    pub destination: String,
    pub created_on: DateTime,
    pub created_by_user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    GlobalRedirectsId,
    PublicRef,
    Alias,
    Destination,
    CreatedOn,
    CreatedByUserId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    GlobalRedirectsId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i32;
    fn auto_increment() -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    ExternalUser,
    GlobalUsage,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::GlobalRedirectsId => ColumnType::Integer.def(),
            Self::PublicRef => ColumnType::String(Some(10u32)).def().unique(),
            Self::Alias => ColumnType::String(Some(512u32)).def().unique(),
            Self::Destination => ColumnType::String(Some(2048u32)).def(),
            Self::CreatedOn => ColumnType::Timestamp.def(),
            Self::CreatedByUserId => ColumnType::Integer.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::ExternalUser => Entity::belongs_to(super::external_user::Entity)
                .from(Column::CreatedByUserId)
                .to(super::external_user::Column::UserId)
                .into(),
            Self::GlobalUsage => Entity::has_many(super::global_usage::Entity).into(),
        }
    }
}

impl Related<super::external_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExternalUser.def()
    }
}

impl Related<super::global_usage::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GlobalUsage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
