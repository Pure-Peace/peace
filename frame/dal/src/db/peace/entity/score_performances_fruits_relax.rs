//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.5

use super::sea_orm_active_enums::PerformanceVersion;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "score_performances_fruits_relax")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub score_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub performance_version: PerformanceVersion,
    #[sea_orm(column_type = "Decimal(Some((16, 2)))")]
    pub performance: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scores_fruits_relax::Entity",
        from = "Column::ScoreId",
        to = "super::scores_fruits_relax::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    ScoresFruitsRelax,
}

impl Related<super::scores_fruits_relax::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ScoresFruitsRelax.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
