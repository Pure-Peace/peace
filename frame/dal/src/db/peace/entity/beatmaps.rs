//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.5

use super::sea_orm_active_enums::GameMode;
use super::sea_orm_active_enums::RankStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "beatmaps")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bid: i32,
    pub sid: i32,
    #[sea_orm(unique)]
    pub md5: String,
    pub title: String,
    pub file_name: String,
    pub artist: String,
    pub diff_name: String,
    pub origin_server: String,
    pub mapper_name: String,
    pub mapper_id: String,
    pub rank_status: RankStatus,
    pub game_mode: GameMode,
    #[sea_orm(column_type = "Decimal(Some((16, 2)))")]
    pub stars: Decimal,
    #[sea_orm(column_type = "Decimal(Some((16, 2)))")]
    pub bpm: Decimal,
    #[sea_orm(column_type = "Decimal(Some((4, 2)))")]
    pub cs: Decimal,
    #[sea_orm(column_type = "Decimal(Some((4, 2)))")]
    pub od: Decimal,
    #[sea_orm(column_type = "Decimal(Some((4, 2)))")]
    pub ar: Decimal,
    #[sea_orm(column_type = "Decimal(Some((4, 2)))")]
    pub hp: Decimal,
    pub length: i32,
    pub length_drain: i32,
    pub source: Option<String>,
    pub tags: Option<String>,
    pub genre_id: Option<i16>,
    pub language_id: Option<i16>,
    pub storyboard: Option<bool>,
    pub video: Option<bool>,
    pub object_count: Option<i32>,
    pub slider_count: Option<i32>,
    pub spinner_count: Option<i32>,
    pub max_combo: Option<i32>,
    pub immutable: bool,
    pub last_update: DateTimeWithTimeZone,
    pub upload_time: DateTimeWithTimeZone,
    pub approved_time: Option<DateTimeWithTimeZone>,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::leaderboard_fruits::Entity")]
    LeaderboardFruits,
    #[sea_orm(has_many = "super::leaderboard_fruits_relax::Entity")]
    LeaderboardFruitsRelax,
    #[sea_orm(has_many = "super::leaderboard_mania::Entity")]
    LeaderboardMania,
    #[sea_orm(has_many = "super::leaderboard_standard::Entity")]
    LeaderboardStandard,
    #[sea_orm(has_many = "super::leaderboard_standard_autopilot::Entity")]
    LeaderboardStandardAutopilot,
    #[sea_orm(has_many = "super::leaderboard_standard_relax::Entity")]
    LeaderboardStandardRelax,
    #[sea_orm(has_many = "super::leaderboard_taiko::Entity")]
    LeaderboardTaiko,
    #[sea_orm(has_many = "super::leaderboard_taiko_relax::Entity")]
    LeaderboardTaikoRelax,
}

impl Related<super::leaderboard_fruits::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardFruits.def()
    }
}

impl Related<super::leaderboard_fruits_relax::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardFruitsRelax.def()
    }
}

impl Related<super::leaderboard_mania::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardMania.def()
    }
}

impl Related<super::leaderboard_standard::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardStandard.def()
    }
}

impl Related<super::leaderboard_standard_autopilot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardStandardAutopilot.def()
    }
}

impl Related<super::leaderboard_standard_relax::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardStandardRelax.def()
    }
}

impl Related<super::leaderboard_taiko::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardTaiko.def()
    }
}

impl Related<super::leaderboard_taiko_relax::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeaderboardTaikoRelax.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::beatmap_ratings::Relation::Users.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::beatmap_ratings::Relation::Beatmaps.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
