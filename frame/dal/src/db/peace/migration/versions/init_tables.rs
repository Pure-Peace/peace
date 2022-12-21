use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum GameMode {
    #[iden = "game_mode"]
    Enum = -1,
    #[iden = "Standard"]
    Standard,
    #[iden = "Taiko"]
    Taiko,
    #[iden = "Fruits"]
    Fruits,
    #[iden = "Mania"]
    Mania,
}

#[derive(Iden)]
pub enum ScoreStatus {
    #[iden = "score_status"]
    Enum = -1,
    #[iden = "Failed"]
    Failed,
    #[iden = "Passed"]
    Passed,
    #[iden = "High"]
    High,
}

#[derive(Iden)]
enum ScoreGrade {
    #[iden = "score_grade"]
    Enum = -1,
    #[iden = "A"]
    A,
    #[iden = "B"]
    B,
    #[iden = "C"]
    C,
    #[iden = "D"]
    D,
    #[iden = "S"]
    S,
    #[iden = "SH"]
    SH,
    #[iden = "X"]
    X,
    #[iden = "XH"]
    XH,
    #[iden = "F"]
    F,
}

#[derive(Iden)]
pub enum RankStatus {
    #[iden = "rank_status"]
    Enum = -3,
    #[iden = "Graveyard"]
    Graveyard,
    #[iden = "Wip"]
    Wip,
    #[iden = "Pending"]
    Pending,
    #[iden = "Ranked"]
    Ranked,
    #[iden = "Approved"]
    Approved,
    #[iden = "Qualified"]
    Qualified,
    #[iden = "Loved"]
    Loved,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let create_table_stmts = vec![
            users::create(),
            bancho_client_hardware_records::create(),
            favourite_beatmaps::create(),
            friend_relationships::create(),
            custom_settings::create(),
            beatmaps::create(),
            beatmap_ratings::create(),
            scores_standard::create(),
            scores_taiko::create(),
            scores_fruits::create(),
            scores_mania::create(),
            scores_standard_relax::create(),
            scores_standard_autopilot::create(),
            scores_taiko_relax::create(),
            scores_fruits_relax::create(),
            user_stats_standard::create(),
            user_stats_taiko::create(),
            user_stats_fruits::create(),
            user_stats_mania::create(),
            user_stats_standard_relax::create(),
            user_stats_standard_autopilot::create(),
            user_stats_taiko_relax::create(),
            user_stats_fruits_relax::create(),
            leaders_standard::create(),
            leaders_taiko::create(),
            leaders_fruits::create(),
            leaders_mania::create(),
            leaders_standard_relax::create(),
            leaders_standard_autopilot::create(),
            leaders_taiko_relax::create(),
            leaders_fruits_relax::create(),
        ];

        let create_foreign_key_stmts = vec![
            bancho_client_hardware_records::create_foreign_keys(),
            favourite_beatmaps::create_foreign_keys(),
            friend_relationships::create_foreign_keys(),
            custom_settings::create_foreign_keys(),
            beatmap_ratings::create_foreign_keys(),
            scores_standard::create_foreign_keys(),
            scores_taiko::create_foreign_keys(),
            scores_fruits::create_foreign_keys(),
            scores_mania::create_foreign_keys(),
            scores_standard_relax::create_foreign_keys(),
            scores_standard_autopilot::create_foreign_keys(),
            scores_taiko_relax::create_foreign_keys(),
            scores_fruits_relax::create_foreign_keys(),
            user_stats_standard::create_foreign_keys(),
            user_stats_taiko::create_foreign_keys(),
            user_stats_fruits::create_foreign_keys(),
            user_stats_mania::create_foreign_keys(),
            user_stats_standard_relax::create_foreign_keys(),
            user_stats_standard_autopilot::create_foreign_keys(),
            user_stats_taiko_relax::create_foreign_keys(),
            user_stats_fruits_relax::create_foreign_keys(),
            leaders_standard::create_foreign_keys(),
            leaders_taiko::create_foreign_keys(),
            leaders_fruits::create_foreign_keys(),
            leaders_mania::create_foreign_keys(),
            leaders_standard_relax::create_foreign_keys(),
            leaders_standard_autopilot::create_foreign_keys(),
            leaders_taiko_relax::create_foreign_keys(),
            leaders_fruits_relax::create_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_index_stmts = vec![
            users::create_indexes(),
            favourite_beatmaps::create_indexes(),
            friend_relationships::create_indexes(),
            beatmaps::create_indexes(),
            beatmap_ratings::create_indexes(),
            scores_standard::create_indexes(),
            scores_taiko::create_indexes(),
            scores_fruits::create_indexes(),
            scores_mania::create_indexes(),
            scores_standard_relax::create_indexes(),
            scores_standard_autopilot::create_indexes(),
            scores_taiko_relax::create_indexes(),
            scores_fruits_relax::create_indexes(),
            leaders_standard::create_indexes(),
            leaders_taiko::create_indexes(),
            leaders_fruits::create_indexes(),
            leaders_mania::create_indexes(),
            leaders_standard_relax::create_indexes(),
            leaders_standard_autopilot::create_indexes(),
            leaders_taiko_relax::create_indexes(),
            leaders_fruits_relax::create_indexes(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_type_stmts = vec![
            extension::postgres::Type::create()
                .as_enum(RankStatus::Enum)
                .values([
                    RankStatus::Graveyard,
                    RankStatus::Wip,
                    RankStatus::Pending,
                    RankStatus::Ranked,
                    RankStatus::Approved,
                    RankStatus::Qualified,
                    RankStatus::Loved,
                ])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(GameMode::Enum)
                .values([
                    GameMode::Standard,
                    GameMode::Taiko,
                    GameMode::Fruits,
                    GameMode::Mania,
                ])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(ScoreStatus::Enum)
                .values([
                    ScoreStatus::Failed,
                    ScoreStatus::Passed,
                    ScoreStatus::High,
                ])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(ScoreGrade::Enum)
                .values([
                    ScoreGrade::A,
                    ScoreGrade::B,
                    ScoreGrade::C,
                    ScoreGrade::D,
                    ScoreGrade::S,
                    ScoreGrade::SH,
                    ScoreGrade::X,
                    ScoreGrade::XH,
                    ScoreGrade::F,
                ])
                .to_owned(),
        ];

        if manager.get_database_backend() == DbBackend::Postgres {
            for stmt in create_type_stmts {
                manager.create_type(stmt).await?;
            }
        }

        for stmt in create_table_stmts {
            manager.create_table(stmt).await?;
        }

        for stmt in create_foreign_key_stmts {
            manager.create_foreign_key(stmt).await?;
        }

        for stmt in create_index_stmts {
            manager.create_index(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_table_stmts = vec![
            users::drop(),
            bancho_client_hardware_records::drop(),
            favourite_beatmaps::drop(),
            friend_relationships::drop(),
            custom_settings::drop(),
            beatmaps::drop(),
            beatmap_ratings::drop(),
            scores_standard::drop(),
            scores_taiko::drop(),
            scores_fruits::drop(),
            scores_mania::drop(),
            scores_standard_relax::drop(),
            scores_standard_autopilot::drop(),
            scores_taiko_relax::drop(),
            scores_fruits_relax::drop(),
            user_stats_standard::drop(),
            user_stats_taiko::drop(),
            user_stats_fruits::drop(),
            user_stats_mania::drop(),
            user_stats_standard_relax::drop(),
            user_stats_standard_autopilot::drop(),
            user_stats_taiko_relax::drop(),
            user_stats_fruits_relax::drop(),
            leaders_standard::drop(),
            leaders_taiko::drop(),
            leaders_fruits::drop(),
            leaders_mania::drop(),
            leaders_standard_relax::drop(),
            leaders_standard_autopilot::drop(),
            leaders_taiko_relax::drop(),
            leaders_fruits_relax::drop(),
        ];

        let drop_foreign_key_stmts = vec![
            bancho_client_hardware_records::drop_foreign_keys(),
            favourite_beatmaps::drop_foreign_keys(),
            friend_relationships::drop_foreign_keys(),
            custom_settings::drop_foreign_keys(),
            beatmap_ratings::drop_foreign_keys(),
            scores_standard::drop_foreign_keys(),
            scores_taiko::drop_foreign_keys(),
            scores_fruits::drop_foreign_keys(),
            scores_mania::drop_foreign_keys(),
            scores_standard_relax::drop_foreign_keys(),
            scores_standard_autopilot::drop_foreign_keys(),
            scores_taiko_relax::drop_foreign_keys(),
            scores_fruits_relax::drop_foreign_keys(),
            user_stats_standard::drop_foreign_keys(),
            user_stats_taiko::drop_foreign_keys(),
            user_stats_fruits::drop_foreign_keys(),
            user_stats_mania::drop_foreign_keys(),
            user_stats_standard_relax::drop_foreign_keys(),
            user_stats_standard_autopilot::drop_foreign_keys(),
            user_stats_taiko_relax::drop_foreign_keys(),
            user_stats_fruits_relax::drop_foreign_keys(),
            leaders_standard::drop_foreign_keys(),
            leaders_taiko::drop_foreign_keys(),
            leaders_fruits::drop_foreign_keys(),
            leaders_mania::drop_foreign_keys(),
            leaders_standard_relax::drop_foreign_keys(),
            leaders_standard_autopilot::drop_foreign_keys(),
            leaders_taiko_relax::drop_foreign_keys(),
            leaders_fruits_relax::drop_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let drop_index_stmts = vec![
            users::drop_indexes(),
            favourite_beatmaps::drop_indexes(),
            friend_relationships::drop_indexes(),
            beatmaps::drop_indexes(),
            beatmap_ratings::drop_indexes(),
            scores_standard::drop_indexes(),
            scores_taiko::drop_indexes(),
            scores_fruits::drop_indexes(),
            scores_mania::drop_indexes(),
            scores_standard_relax::drop_indexes(),
            scores_standard_autopilot::drop_indexes(),
            scores_taiko_relax::drop_indexes(),
            scores_fruits_relax::drop_indexes(),
            leaders_standard::drop_indexes(),
            leaders_taiko::drop_indexes(),
            leaders_fruits::drop_indexes(),
            leaders_mania::drop_indexes(),
            leaders_standard_relax::drop_indexes(),
            leaders_standard_autopilot::drop_indexes(),
            leaders_taiko_relax::drop_indexes(),
            leaders_fruits_relax::drop_indexes(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let drop_type_stmts = vec![
            extension::postgres::Type::drop().name(RankStatus::Enum).to_owned(),
            extension::postgres::Type::drop().name(GameMode::Enum).to_owned(),
            extension::postgres::Type::drop()
                .name(ScoreStatus::Enum)
                .to_owned(),
            extension::postgres::Type::drop().name(ScoreGrade::Enum).to_owned(),
        ];

        for stmt in drop_foreign_key_stmts {
            manager.drop_foreign_key(stmt).await?;
        }

        for stmt in drop_index_stmts {
            manager.drop_index(stmt).await?;
        }

        for stmt in drop_table_stmts {
            manager.drop_table(stmt).await?;
        }

        if manager.get_database_backend() == DbBackend::Postgres {
            for stmt in drop_type_stmts {
                manager.drop_type(stmt).await?;
            }
        }

        Ok(())
    }
}

pub mod users {
    use sea_orm_migration::prelude::*;

    const INDEX_NAME_SAFE: &str = "IDX_users_name_safe";
    const INDEX_NAME_UNICODE_SAFE: &str = "IDX_users_name_unicode_safe";
    const INDEX_EMAIL: &str = "IDX_users_email";

    #[derive(Iden)]
    pub enum Users {
        Table,
        Id,
        Name,
        NameSafe,
        NameUnicode,
        NameUnicodeSafe,
        Password,
        Email,
        Privileges,
        Country,
        CreatedAt,
        UpdatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(Users::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Users::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Users::Name)
                    .string()
                    .string_len(16)
                    .unique_key()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Users::NameSafe)
                    .string()
                    .string_len(16)
                    .unique_key()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Users::NameUnicode)
                    .string()
                    .string_len(10)
                    .unique_key()
                    .null(),
            )
            .col(
                ColumnDef::new(Users::NameUnicodeSafe)
                    .string()
                    .string_len(10)
                    .unique_key()
                    .null(),
            )
            .col(ColumnDef::new(Users::Password).string().not_null())
            .col(
                ColumnDef::new(Users::Email)
                    .string()
                    .string_len(64)
                    .unique_key()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Users::Privileges)
                    .integer()
                    .default(1)
                    .not_null(),
            )
            .col(ColumnDef::new(Users::Country).string().string_len(8).null())
            .col(
                ColumnDef::new(Users::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Users::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(Users::Table).to_owned()
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![
            sea_query::Index::create()
                .name(INDEX_NAME_SAFE)
                .table(Users::Table)
                .col(Users::NameSafe)
                .unique()
                .to_owned(),
            sea_query::Index::create()
                .name(INDEX_NAME_UNICODE_SAFE)
                .table(Users::Table)
                .col(Users::NameUnicodeSafe)
                .unique()
                .to_owned(),
            sea_query::Index::create()
                .name(INDEX_EMAIL)
                .table(Users::Table)
                .col(Users::Email)
                .unique()
                .to_owned(),
        ]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![
            sea_query::Index::drop()
                .table(Users::Table)
                .name(INDEX_NAME_SAFE)
                .to_owned(),
            sea_query::Index::drop()
                .table(Users::Table)
                .name(INDEX_NAME_UNICODE_SAFE)
                .to_owned(),
            sea_query::Index::drop()
                .table(Users::Table)
                .name(INDEX_EMAIL)
                .to_owned(),
        ]
    }
}

pub mod bancho_client_hardware_records {
    use sea_orm_migration::prelude::*;

    use super::users::Users;

    const FOREIGN_KEY_USER_ID: &str =
        "FK_bancho_client_hardware_records_user_id";

    #[derive(Iden)]
    enum BanchoClientHardwareRecords {
        Table,
        UserId,
        TimeOffset,
        PathHash,
        Adapters,
        AdaptersHash,
        UninstallId,
        DiskId,
        UsedTimes,
        CreatedAt,
        UpdatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(BanchoClientHardwareRecords::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::UserId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::TimeOffset)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::PathHash)
                    .char()
                    .char_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::Adapters)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::AdaptersHash)
                    .char()
                    .char_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::UninstallId)
                    .char()
                    .char_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::DiskId)
                    .char()
                    .char_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::UsedTimes)
                    .integer()
                    .default(1)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(BanchoClientHardwareRecords::UserId)
                    .col(BanchoClientHardwareRecords::PathHash)
                    .col(BanchoClientHardwareRecords::AdaptersHash)
                    .col(BanchoClientHardwareRecords::UninstallId)
                    .col(BanchoClientHardwareRecords::DiskId),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(BanchoClientHardwareRecords::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![sea_query::ForeignKey::create()
            .name(FOREIGN_KEY_USER_ID)
            .from(
                BanchoClientHardwareRecords::Table,
                BanchoClientHardwareRecords::UserId,
            )
            .to(Users::Table, Users::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned()]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![sea_query::ForeignKey::drop()
            .name(FOREIGN_KEY_USER_ID)
            .table(BanchoClientHardwareRecords::Table)
            .to_owned()]
    }
}

pub mod favourite_beatmaps {
    use sea_orm_migration::prelude::*;

    use super::users::Users;

    const FOREIGN_KEY_USER_ID: &str = "FK_favourite_beatmaps_user_id";
    const INDEX_USER_ID: &str = "IDX_favourite_beatmaps_user_id";

    #[derive(Iden)]
    pub enum FavouriteBeatmaps {
        Table,
        UserId,
        MapId,
        Comment,
        CreatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(FavouriteBeatmaps::Table)
            .if_not_exists()
            .col(ColumnDef::new(FavouriteBeatmaps::UserId).integer().not_null())
            .col(ColumnDef::new(FavouriteBeatmaps::MapId).integer().not_null())
            .col(
                ColumnDef::new(FavouriteBeatmaps::Comment)
                    .string()
                    .string_len(15)
                    .null(),
            )
            .col(
                ColumnDef::new(FavouriteBeatmaps::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(FavouriteBeatmaps::UserId)
                    .col(FavouriteBeatmaps::MapId),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(FavouriteBeatmaps::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![sea_query::ForeignKey::create()
            .name(FOREIGN_KEY_USER_ID)
            .from(FavouriteBeatmaps::Table, FavouriteBeatmaps::UserId)
            .to(Users::Table, Users::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned()]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![sea_query::ForeignKey::drop()
            .name(FOREIGN_KEY_USER_ID)
            .table(FavouriteBeatmaps::Table)
            .to_owned()]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_USER_ID)
            .table(FavouriteBeatmaps::Table)
            .col(FavouriteBeatmaps::UserId)
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(FavouriteBeatmaps::Table)
            .name(INDEX_USER_ID)
            .to_owned()]
    }
}

pub mod friend_relationships {
    use sea_orm_migration::prelude::*;

    use super::users::Users;

    const FOREIGN_KEY_USER_ID: &str = "FK_friend_relationships_user_id";
    const FOREIGN_KEY_FRIEND_ID: &str = "FK_friend_relationships_friend_id";
    const INDEX_USER_ID: &str = "IDX_friend_relationships_user_id";

    #[derive(Iden)]
    pub enum FriendRelationships {
        Table,
        UserId,
        FriendId,
        Remark,
        CreatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(FriendRelationships::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(FriendRelationships::UserId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(FriendRelationships::FriendId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(FriendRelationships::Remark)
                    .string()
                    .string_len(16)
                    .null(),
            )
            .col(
                ColumnDef::new(FriendRelationships::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(FriendRelationships::UserId)
                    .col(FriendRelationships::FriendId),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(FriendRelationships::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_USER_ID)
                .from(FriendRelationships::Table, FriendRelationships::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_FRIEND_ID)
                .from(FriendRelationships::Table, FriendRelationships::FriendId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
        ]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_USER_ID)
                .table(FriendRelationships::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_FRIEND_ID)
                .table(FriendRelationships::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_USER_ID)
            .table(FriendRelationships::Table)
            .col(FriendRelationships::UserId)
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(FriendRelationships::Table)
            .name(INDEX_USER_ID)
            .to_owned()]
    }
}

pub mod custom_settings {
    use sea_orm_migration::prelude::*;

    use super::users::Users;

    const FOREIGN_KEY_USER_ID: &str = "FK_custom_settings_user_id";

    #[derive(Iden)]
    pub enum CustomSettings {
        Table,
        UserId,
        DisplayUnicodeName,
        PpScoreboard,
        InvisibleOnline,
        UpdatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(CustomSettings::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(CustomSettings::UserId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(CustomSettings::DisplayUnicodeName)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(CustomSettings::PpScoreboard)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(CustomSettings::InvisibleOnline)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(CustomSettings::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(CustomSettings::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![sea_query::ForeignKey::create()
            .name(FOREIGN_KEY_USER_ID)
            .from(CustomSettings::Table, CustomSettings::UserId)
            .to(Users::Table, Users::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned()]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![sea_query::ForeignKey::drop()
            .name(FOREIGN_KEY_USER_ID)
            .table(CustomSettings::Table)
            .to_owned()]
    }
}

pub mod beatmaps {
    use sea_orm_migration::prelude::*;

    use super::{GameMode, RankStatus};

    const INDEX_SID: &str = "IDX_beatmaps_sid";
    const INDEX_MD5: &str = "IDX_beatmaps_md5";
    const INDEX_FILE_NAME: &str = "IDX_beatmaps_file_name";
    const INDEX_RANK_STATUS: &str = "IDX_beatmaps_rank_status";

    #[derive(Iden)]
    pub enum Beatmaps {
        Table,
        Bid,
        Sid,
        Md5,
        Title,
        FileName,
        Artist,
        DiffName,
        OriginServer,
        MapperName,
        MapperId,
        RankStatus,
        GameMode,
        Stars,
        Bpm,
        Cs,
        Od,
        Ar,
        Hp,
        Length,
        LengthDrain,
        Source,
        Tags,
        GenreId,
        LanguageId,
        Storyboard,
        Video,
        ObjectCount,
        SliderCount,
        SpinnerCount,
        MaxCombo,
        Immutable,
        LastUpdate,
        UploadTime,
        ApprovedTime,
        UpdatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(Beatmaps::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Beatmaps::Bid)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(Beatmaps::Sid).integer().not_null())
            .col(
                ColumnDef::new(Beatmaps::Md5)
                    .char()
                    .char_len(32)
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(Beatmaps::Title).string().not_null())
            .col(ColumnDef::new(Beatmaps::FileName).string().not_null())
            .col(ColumnDef::new(Beatmaps::Artist).string().not_null())
            .col(ColumnDef::new(Beatmaps::DiffName).string().not_null())
            .col(ColumnDef::new(Beatmaps::OriginServer).string().not_null())
            .col(ColumnDef::new(Beatmaps::MapperName).string().not_null())
            .col(ColumnDef::new(Beatmaps::MapperId).string().not_null())
            .col(
                ColumnDef::new(Beatmaps::RankStatus)
                    .enumeration(
                        RankStatus::Enum,
                        [
                            RankStatus::Graveyard,
                            RankStatus::Wip,
                            RankStatus::Pending,
                            RankStatus::Ranked,
                            RankStatus::Approved,
                            RankStatus::Qualified,
                            RankStatus::Loved,
                        ],
                    )
                    .not_null()
                    .default(RankStatus::Pending.to_string()),
            )
            .col(
                ColumnDef::new(Beatmaps::GameMode)
                    .enumeration(
                        GameMode::Enum,
                        [
                            GameMode::Standard,
                            GameMode::Taiko,
                            GameMode::Fruits,
                            GameMode::Mania,
                        ],
                    )
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::Stars)
                    .decimal()
                    .decimal_len(16, 2)
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::Bpm)
                    .decimal()
                    .decimal_len(16, 2)
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::Cs)
                    .decimal()
                    .decimal_len(4, 2)
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::Od)
                    .decimal()
                    .decimal_len(4, 2)
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::Ar)
                    .decimal()
                    .decimal_len(4, 2)
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::Hp)
                    .decimal()
                    .decimal_len(4, 2)
                    .not_null(),
            )
            .col(ColumnDef::new(Beatmaps::Length).integer().not_null())
            .col(ColumnDef::new(Beatmaps::LengthDrain).integer().not_null())
            .col(ColumnDef::new(Beatmaps::Source).string().null())
            .col(ColumnDef::new(Beatmaps::Tags).string().null())
            .col(ColumnDef::new(Beatmaps::GenreId).small_integer().null())
            .col(ColumnDef::new(Beatmaps::LanguageId).small_integer().null())
            .col(ColumnDef::new(Beatmaps::Storyboard).boolean().null())
            .col(ColumnDef::new(Beatmaps::Video).boolean().null())
            .col(ColumnDef::new(Beatmaps::ObjectCount).integer().null())
            .col(ColumnDef::new(Beatmaps::SliderCount).integer().null())
            .col(ColumnDef::new(Beatmaps::SpinnerCount).integer().null())
            .col(ColumnDef::new(Beatmaps::MaxCombo).integer().null())
            .col(
                ColumnDef::new(Beatmaps::Immutable)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(Beatmaps::LastUpdate)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::UploadTime)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::ApprovedTime)
                    .timestamp_with_time_zone()
                    .null(),
            )
            .col(
                ColumnDef::new(Beatmaps::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(Beatmaps::Table).to_owned()
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![
            sea_query::Index::create()
                .name(INDEX_SID)
                .table(Beatmaps::Table)
                .col(Beatmaps::Sid)
                .to_owned(),
            sea_query::Index::create()
                .name(INDEX_MD5)
                .table(Beatmaps::Table)
                .col(Beatmaps::Md5)
                .unique()
                .to_owned(),
            sea_query::Index::create()
                .name(INDEX_FILE_NAME)
                .table(Beatmaps::Table)
                .col(Beatmaps::FileName)
                .to_owned(),
            sea_query::Index::create()
                .name(INDEX_RANK_STATUS)
                .table(Beatmaps::Table)
                .col(Beatmaps::RankStatus)
                .to_owned(),
        ]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![
            sea_query::Index::drop()
                .table(Beatmaps::Table)
                .name(INDEX_SID)
                .to_owned(),
            sea_query::Index::drop()
                .table(Beatmaps::Table)
                .name(INDEX_MD5)
                .to_owned(),
            sea_query::Index::drop()
                .table(Beatmaps::Table)
                .name(INDEX_FILE_NAME)
                .to_owned(),
            sea_query::Index::drop()
                .table(Beatmaps::Table)
                .name(INDEX_RANK_STATUS)
                .to_owned(),
        ]
    }
}

pub mod beatmap_ratings {
    use sea_orm_migration::prelude::*;

    use super::{beatmaps::Beatmaps, users::Users};

    const FOREIGN_KEY_USER_ID: &str = "FK_beatmap_ratings_user_id";
    const FOREIGN_KEY_MAP_MD5: &str = "FK_beatmap_ratings_map_md5";
    const INDEX_MD5: &str = "IDX_beatmap_ratings_map_md5";

    #[derive(Iden)]
    pub enum BeatmapRatings {
        Table,
        UserId,
        MapMd5,
        Rating,
        UpdatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(BeatmapRatings::Table)
            .if_not_exists()
            .col(ColumnDef::new(BeatmapRatings::UserId).integer().not_null())
            .col(
                ColumnDef::new(BeatmapRatings::MapMd5)
                    .char()
                    .char_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BeatmapRatings::Rating)
                    .small_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BeatmapRatings::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(BeatmapRatings::UserId)
                    .col(BeatmapRatings::MapMd5),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(BeatmapRatings::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_USER_ID)
                .from(BeatmapRatings::Table, BeatmapRatings::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_MAP_MD5)
                .from(BeatmapRatings::Table, BeatmapRatings::MapMd5)
                .to(Beatmaps::Table, Beatmaps::Md5)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
        ]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_USER_ID)
                .table(BeatmapRatings::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_MAP_MD5)
                .table(BeatmapRatings::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_MD5)
            .table(BeatmapRatings::Table)
            .col(BeatmapRatings::MapMd5)
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(BeatmapRatings::Table)
            .name(INDEX_MD5)
            .to_owned()]
    }
}

macro_rules! define_scores {
    ($table_name: ident, $iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::{users::Users, ScoreGrade, ScoreStatus};

            const FOREIGN_KEY_USER_ID: &str =
                concat!("FK_", stringify!($table_name), "_user_id");
            const INDEX_MAP_MD5: &str =
                concat!("IDX_", stringify!($table_name), "_map_md5");
            const INDEX_USER_ID: &str =
                concat!("IDX_", stringify!($table_name), "_user_id");

            #[derive(Iden)]
            pub enum $iden {
                Table,
                Id,
                UserId,
                ScoreMd5,
                MapMd5,
                Score,
                Performance,
                Accuracy,
                Combo,
                Mods,
                N300,
                N100,
                N50,
                Miss,
                Geki,
                Katu,
                Playtime,
                Perfect,
                Status,
                Grade,
                ClientFlags,
                ClientVersion,
                Confidence,
                Verified,
                Invisible,
                VerifyAt,
                CreateAt,
                UpdatedAt,
            }

            pub fn create() -> TableCreateStatement {
                Table::create()
                    .table($iden::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new($iden::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new($iden::UserId).integer().not_null())
                    .col(
                        ColumnDef::new($iden::ScoreMd5)
                            .char()
                            .char_len(32)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new($iden::MapMd5)
                            .char()
                            .char_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new($iden::Score).integer().not_null())
                    .col(
                        ColumnDef::new($iden::Performance)
                            .decimal()
                            .decimal_len(16, 2)
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new($iden::Accuracy)
                            .decimal()
                            .decimal_len(6, 2)
                            .not_null(),
                    )
                    .col(ColumnDef::new($iden::Combo).integer().not_null())
                    .col(ColumnDef::new($iden::Mods).integer().not_null())
                    .col(ColumnDef::new($iden::N300).integer().not_null())
                    .col(ColumnDef::new($iden::N100).integer().not_null())
                    .col(ColumnDef::new($iden::N50).integer().not_null())
                    .col(ColumnDef::new($iden::Miss).integer().not_null())
                    .col(ColumnDef::new($iden::Geki).integer().not_null())
                    .col(ColumnDef::new($iden::Katu).integer().not_null())
                    .col(ColumnDef::new($iden::Playtime).integer().not_null())
                    .col(
                        ColumnDef::new($iden::Perfect)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new($iden::Status)
                            .enumeration(
                                ScoreStatus::Enum,
                                [
                                    ScoreStatus::Failed,
                                    ScoreStatus::Passed,
                                    ScoreStatus::High,
                                ],
                            )
                            .not_null()
                            .default(ScoreStatus::Failed.to_string()),
                    )
                    .col(
                        ColumnDef::new($iden::Grade)
                            .enumeration(
                                ScoreGrade::Enum,
                                [
                                    ScoreGrade::A,
                                    ScoreGrade::B,
                                    ScoreGrade::C,
                                    ScoreGrade::D,
                                    ScoreGrade::S,
                                    ScoreGrade::SH,
                                    ScoreGrade::X,
                                    ScoreGrade::XH,
                                    ScoreGrade::F,
                                ],
                            )
                            .not_null()
                            .default(ScoreGrade::F.to_string()),
                    )
                    .col(
                        ColumnDef::new($iden::ClientFlags).integer().not_null(),
                    )
                    .col(
                        ColumnDef::new($iden::ClientVersion)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new($iden::Confidence).integer().null())
                    .col(
                        ColumnDef::new($iden::Verified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new($iden::Invisible)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new($iden::VerifyAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new($iden::CreateAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new($iden::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned()
            }

            pub fn drop() -> TableDropStatement {
                Table::drop().table($iden::Table).to_owned()
            }

            pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
                vec![sea_query::ForeignKey::create()
                    .name(FOREIGN_KEY_USER_ID)
                    .from($iden::Table, $iden::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned()]
            }

            pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
                vec![sea_query::ForeignKey::drop()
                    .name(FOREIGN_KEY_USER_ID)
                    .table($iden::Table)
                    .to_owned()]
            }

            pub fn create_indexes() -> Vec<IndexCreateStatement> {
                vec![
                    sea_query::Index::create()
                        .name(INDEX_MAP_MD5)
                        .table($iden::Table)
                        .col($iden::MapMd5)
                        .to_owned(),
                    sea_query::Index::create()
                        .name(INDEX_USER_ID)
                        .table($iden::Table)
                        .col($iden::UserId)
                        .to_owned(),
                ]
            }

            pub fn drop_indexes() -> Vec<IndexDropStatement> {
                vec![
                    sea_query::Index::drop()
                        .table($iden::Table)
                        .name(INDEX_MAP_MD5)
                        .to_owned(),
                    sea_query::Index::drop()
                        .table($iden::Table)
                        .name(INDEX_USER_ID)
                        .to_owned(),
                ]
            }
        }
    };
}

define_scores!(scores_standard, ScoresStandard);
define_scores!(scores_taiko, ScoresTaiko);
define_scores!(scores_fruits, ScoresFruits);
define_scores!(scores_mania, ScoresMania);
define_scores!(scores_standard_relax, ScoresStandardRelax);
define_scores!(scores_standard_autopilot, ScoresStandardAutopilot);
define_scores!(scores_taiko_relax, ScoresTaikoRelax);
define_scores!(scores_fruits_relax, ScoresFruitsRelax);

macro_rules! define_user_stats {
    ($table_name: ident, $iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::users::Users;

            const FOREIGN_KEY_USER_ID: &str =
                concat!("FK_", stringify!($table_name), "_user_id");

            #[derive(Iden)]
            pub enum $iden {
                Table,
                UserId,
                TotalScore,
                RankedScore,
                Performance,
                Playcount,
                TotalHits,
                Accuracy,
                MaxCombo,
                TotalSecondsPlayed,
                Count300,
                Count100,
                Count50,
                CountMiss,
                CountFailed,
                CountQuit,
                UpdatedAt,
            }

            pub fn create() -> TableCreateStatement {
                Table::create()
                    .table($iden::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new($iden::UserId)
                            .integer()
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new($iden::TotalScore)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::RankedScore)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::Performance)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::Playcount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::TotalHits)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::Accuracy)
                            .decimal()
                            .decimal_len(6, 2)
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new($iden::MaxCombo)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::TotalSecondsPlayed)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::Count300)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::Count100)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::Count50)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::CountMiss)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::CountFailed)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::CountQuit)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new($iden::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned()
            }

            pub fn drop() -> TableDropStatement {
                Table::drop().table($iden::Table).to_owned()
            }

            pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
                vec![sea_query::ForeignKey::create()
                    .name(FOREIGN_KEY_USER_ID)
                    .from($iden::Table, $iden::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned()]
            }

            pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
                vec![sea_query::ForeignKey::drop()
                    .name(FOREIGN_KEY_USER_ID)
                    .table($iden::Table)
                    .to_owned()]
            }
        }
    };
}

define_user_stats!(user_stats_standard, UserStatsStandard);
define_user_stats!(user_stats_taiko, UserStatsTaiko);
define_user_stats!(user_stats_fruits, UserStatsFruits);
define_user_stats!(user_stats_mania, UserStatsMania);
define_user_stats!(user_stats_standard_relax, UserStatsStandardRelax);
define_user_stats!(user_stats_standard_autopilot, UserStatsStandardAutopilot);
define_user_stats!(user_stats_taiko_relax, UserStatsTaikoRelax);
define_user_stats!(user_stats_fruits_relax, UserStatsFruitsRelax);

macro_rules! define_leaders {
    ($table_name: ident, $iden: ident, $relate_table: ident :: $relate_iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::users::Users;
            use super::$relate_table::$relate_iden;

            const FOREIGN_KEY_USER_ID: &str =
                concat!("FK_", stringify!($table_name), "_user_id");
            const FOREIGN_KEY_SCORE_ID: &str =
                concat!("FK_", stringify!($table_name), "_score_id");
            const INDEX_USER_ID: &str =
                concat!("IDX_", stringify!($table_name), "_user_id");
            const INDEX_SCORE_ID: &str =
                concat!("IDX_", stringify!($table_name), "_score_id");

            #[derive(Iden)]
            pub enum $iden {
                Table,
                BeatmapId,
                UserId,
                ScoreId,
            }

            pub fn create() -> TableCreateStatement {
                Table::create()
                    .table($iden::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new($iden::BeatmapId)
                            .integer()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new($iden::UserId).integer().not_null())
                    .col(
                        ColumnDef::new($iden::ScoreId).big_integer().not_null(),
                    )
                    .to_owned()
            }

            pub fn drop() -> TableDropStatement {
                Table::drop().table($iden::Table).to_owned()
            }

            pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
                vec![
                    sea_query::ForeignKey::create()
                        .name(FOREIGN_KEY_USER_ID)
                        .from($iden::Table, $iden::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        .to_owned(),
                    sea_query::ForeignKey::create()
                        .name(FOREIGN_KEY_SCORE_ID)
                        .from($iden::Table, $iden::ScoreId)
                        .to($relate_iden::Table, $relate_iden::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        .to_owned(),
                ]
            }

            pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
                vec![
                    sea_query::ForeignKey::drop()
                        .name(FOREIGN_KEY_USER_ID)
                        .table($iden::Table)
                        .to_owned(),
                    sea_query::ForeignKey::drop()
                        .name(FOREIGN_KEY_SCORE_ID)
                        .table($iden::Table)
                        .to_owned(),
                ]
            }

            pub fn create_indexes() -> Vec<IndexCreateStatement> {
                vec![
                    sea_query::Index::create()
                        .name(INDEX_USER_ID)
                        .table($iden::Table)
                        .col($iden::UserId)
                        .to_owned(),
                    sea_query::Index::create()
                        .name(INDEX_SCORE_ID)
                        .table($iden::Table)
                        .col($iden::ScoreId)
                        .to_owned(),
                ]
            }

            pub fn drop_indexes() -> Vec<IndexDropStatement> {
                vec![
                    sea_query::Index::drop()
                        .table($iden::Table)
                        .name(INDEX_USER_ID)
                        .to_owned(),
                    sea_query::Index::drop()
                        .table($iden::Table)
                        .name(INDEX_SCORE_ID)
                        .to_owned(),
                ]
            }
        }
    };
}

define_leaders!(
    leaders_standard,
    LeadersStandard,
    scores_standard::ScoresStandard
);
define_leaders!(leaders_taiko, LeadersTaiko, scores_taiko::ScoresTaiko);
define_leaders!(leaders_fruits, LeadersFruits, scores_fruits::ScoresFruits);
define_leaders!(leaders_mania, LeadersMania, scores_mania::ScoresMania);
define_leaders!(
    leaders_standard_relax,
    LeadersStandardRelax,
    scores_standard_relax::ScoresStandardRelax
);
define_leaders!(
    leaders_standard_autopilot,
    LeadersStandardAutopilot,
    scores_standard_autopilot::ScoresStandardAutopilot
);
define_leaders!(
    leaders_taiko_relax,
    LeadersTaikoRelax,
    scores_taiko_relax::ScoresTaikoRelax
);
define_leaders!(
    leaders_fruits_relax,
    LeadersFruitsRelax,
    scores_fruits_relax::ScoresFruitsRelax
);
