//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "leaders_fruits")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub beatmap_id: i32,
    pub user_id: i32,
    pub score_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scores_fruits::Entity",
        from = "Column::ScoreId",
        to = "super::scores_fruits::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ScoresFruits,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::scores_fruits::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ScoresFruits.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
