//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use super::sea_orm_active_enums::ChannelHandleType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "channel_privileges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub channel_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub handle: ChannelHandleType,
    pub required_privilege_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::channels::Entity",
        from = "Column::ChannelId",
        to = "super::channels::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Channels,
    #[sea_orm(
        belongs_to = "super::privileges::Entity",
        from = "Column::RequiredPrivilegeId",
        to = "super::privileges::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Privileges,
}

impl Related<super::channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channels.def()
    }
}

impl Related<super::privileges::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Privileges.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}