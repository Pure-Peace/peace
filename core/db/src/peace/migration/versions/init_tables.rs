use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum GameMode {
    #[iden = "game_mode"]
    Enum = -1,
    #[iden = "Standard"]
    Standard = 0,
    #[iden = "Taiko"]
    Taiko = 1,
    #[iden = "Fruits"]
    Fruits = 2,
    #[iden = "Mania"]
    Mania = 3,
}

#[derive(Iden)]
pub enum ScoreStatus {
    #[iden = "score_status"]
    Enum = -1,
    #[iden = "Failed"]
    Failed = 0,
    #[iden = "Passed"]
    Passed = 1,
    #[iden = "High"]
    High = 2,
}

#[derive(Iden)]
enum ScoreGrade {
    #[iden = "score_grade"]
    Enum = -1,
    #[iden = "A"]
    A = 0,
    #[iden = "B"]
    B = 1,
    #[iden = "C"]
    C = 2,
    #[iden = "D"]
    D = 3,
    #[iden = "S"]
    S = 4,
    #[iden = "SH"]
    SH = 5,
    #[iden = "X"]
    X = 6,
    #[iden = "XH"]
    XH = 7,
    #[iden = "F"]
    F = 8,
}

#[derive(Iden)]
pub enum RankStatus {
    #[iden = "rank_status"]
    Enum = -3,
    #[iden = "Graveyard"]
    Graveyard = -2,
    #[iden = "Wip"]
    Wip = -1,
    #[iden = "Pending"]
    Pending = 0,
    #[iden = "Ranked"]
    Ranked = 1,
    #[iden = "Approved"]
    Approved = 2,
    #[iden = "Qualified"]
    Qualified = 3,
    #[iden = "Loved"]
    Loved = 4,
}

#[derive(Iden)]
pub enum PPVersion {
    #[iden = "pp_version"]
    Enum = -1,
    #[iden = "v1"]
    V1 = 0,
    #[iden = "v2"]
    V2 = 1,
}

#[derive(Iden)]
pub enum ScoreVersion {
    #[iden = "score_version"]
    Enum = -1,
    #[iden = "v1"]
    V1 = 0,
    #[iden = "v2"]
    V2 = 1,
}

#[derive(Iden)]
pub enum RankingType {
    #[iden = "ranking_type"]
    Enum = -1,
    #[iden = "score_v1"]
    ScoreV1 = 0,
    #[iden = "score_v2"]
    ScoreV2 = 1,
    #[iden = "pp_v1"]
    PPV1 = 2,
    #[iden = "pp_v2"]
    PPV2 = 3,
}

#[derive(Iden)]
pub enum ChannelType {
    #[iden = "channel_type"]
    Enum = -1,
    #[iden = "private"]
    Private = 0,
    #[iden = "public"]
    Public = 1,
    #[iden = "group"]
    Group = 2,
    #[iden = "multiplayer"]
    Multiplayer = 3,
    #[iden = "spectaor"]
    Spectaor = 4,
}

#[derive(Iden)]
pub enum ChannelHandleType {
    #[iden = "channel_handle_type"]
    Enum = -1,
    #[iden = "join"]
    Join = 0,
    #[iden = "send_message"]
    SendMessage = 1,
    #[iden = "kick_user"]
    KickUser = 2,
    #[iden = "mute_user"]
    MuteUser = 3,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let create_table_stmts = vec![
            users::create(),
            privileges::create(),
            user_privileges::create(),
            bancho_client_hardware_records::create(),
            favourite_beatmaps::create(),
            followers::create(),
            user_settings::create(),
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
            score_pp_standard::create(),
            score_pp_taiko::create(),
            score_pp_fruits::create(),
            score_pp_mania::create(),
            score_pp_standard_relax::create(),
            score_pp_standard_autopilot::create(),
            score_pp_taiko_relax::create(),
            score_pp_fruits_relax::create(),
            user_stats_standard::create(),
            user_stats_standard_score_v2::create(),
            user_stats_taiko::create(),
            user_stats_fruits::create(),
            user_stats_mania::create(),
            user_stats_standard_relax::create(),
            user_stats_standard_autopilot::create(),
            user_stats_taiko_relax::create(),
            user_stats_fruits_relax::create(),
            user_pp_standard::create(),
            user_pp_taiko::create(),
            user_pp_fruits::create(),
            user_pp_mania::create(),
            user_pp_standard_relax::create(),
            user_pp_standard_autopilot::create(),
            user_pp_taiko_relax::create(),
            user_pp_fruits_relax::create(),
            leaderboard_standard::create(),
            leaderboard_taiko::create(),
            leaderboard_fruits::create(),
            leaderboard_mania::create(),
            leaderboard_standard_relax::create(),
            leaderboard_standard_autopilot::create(),
            leaderboard_taiko_relax::create(),
            leaderboard_fruits_relax::create(),
            channels::create(),
            channel_users::create(),
            channel_privileges::create(),
            chat_messages::create(),
        ];

        let create_foreign_key_stmts = vec![
            user_privileges::create_foreign_keys(),
            bancho_client_hardware_records::create_foreign_keys(),
            favourite_beatmaps::create_foreign_keys(),
            followers::create_foreign_keys(),
            user_settings::create_foreign_keys(),
            beatmap_ratings::create_foreign_keys(),
            scores_standard::create_foreign_keys(),
            scores_taiko::create_foreign_keys(),
            scores_fruits::create_foreign_keys(),
            scores_mania::create_foreign_keys(),
            scores_standard_relax::create_foreign_keys(),
            scores_standard_autopilot::create_foreign_keys(),
            scores_taiko_relax::create_foreign_keys(),
            scores_fruits_relax::create_foreign_keys(),
            score_pp_standard::create_foreign_keys(),
            score_pp_taiko::create_foreign_keys(),
            score_pp_fruits::create_foreign_keys(),
            score_pp_mania::create_foreign_keys(),
            score_pp_standard_relax::create_foreign_keys(),
            score_pp_standard_autopilot::create_foreign_keys(),
            score_pp_taiko_relax::create_foreign_keys(),
            score_pp_fruits_relax::create_foreign_keys(),
            user_stats_standard::create_foreign_keys(),
            user_stats_standard_score_v2::create_foreign_keys(),
            user_stats_taiko::create_foreign_keys(),
            user_stats_fruits::create_foreign_keys(),
            user_stats_mania::create_foreign_keys(),
            user_stats_standard_relax::create_foreign_keys(),
            user_stats_standard_autopilot::create_foreign_keys(),
            user_stats_taiko_relax::create_foreign_keys(),
            user_stats_fruits_relax::create_foreign_keys(),
            user_pp_standard::create_foreign_keys(),
            user_pp_taiko::create_foreign_keys(),
            user_pp_fruits::create_foreign_keys(),
            user_pp_mania::create_foreign_keys(),
            user_pp_standard_relax::create_foreign_keys(),
            user_pp_standard_autopilot::create_foreign_keys(),
            user_pp_taiko_relax::create_foreign_keys(),
            user_pp_fruits_relax::create_foreign_keys(),
            leaderboard_standard::create_foreign_keys(),
            leaderboard_taiko::create_foreign_keys(),
            leaderboard_fruits::create_foreign_keys(),
            leaderboard_mania::create_foreign_keys(),
            leaderboard_standard_relax::create_foreign_keys(),
            leaderboard_standard_autopilot::create_foreign_keys(),
            leaderboard_taiko_relax::create_foreign_keys(),
            leaderboard_fruits_relax::create_foreign_keys(),
            channel_users::create_foreign_keys(),
            channel_privileges::create_foreign_keys(),
            chat_messages::create_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_index_stmts = vec![
            users::create_indexes(),
            privileges::create_indexes(),
            user_privileges::create_indexes(),
            favourite_beatmaps::create_indexes(),
            followers::create_indexes(),
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
            score_pp_standard::create_indexes(),
            score_pp_taiko::create_indexes(),
            score_pp_fruits::create_indexes(),
            score_pp_mania::create_indexes(),
            score_pp_standard_relax::create_indexes(),
            score_pp_standard_autopilot::create_indexes(),
            score_pp_taiko_relax::create_indexes(),
            score_pp_fruits_relax::create_indexes(),
            user_pp_standard::create_indexes(),
            user_pp_taiko::create_indexes(),
            user_pp_fruits::create_indexes(),
            user_pp_mania::create_indexes(),
            user_pp_standard_relax::create_indexes(),
            user_pp_standard_autopilot::create_indexes(),
            user_pp_taiko_relax::create_indexes(),
            user_pp_fruits_relax::create_indexes(),
            leaderboard_standard::create_indexes(),
            leaderboard_taiko::create_indexes(),
            leaderboard_fruits::create_indexes(),
            leaderboard_mania::create_indexes(),
            leaderboard_standard_relax::create_indexes(),
            leaderboard_standard_autopilot::create_indexes(),
            leaderboard_taiko_relax::create_indexes(),
            leaderboard_fruits_relax::create_indexes(),
            channels::create_indexes(),
            channel_users::create_indexes(),
            channel_privileges::create_indexes(),
            chat_messages::create_indexes(),
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
            extension::postgres::Type::create()
                .as_enum(PPVersion::Enum)
                .values([PPVersion::V1, PPVersion::V2])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(ScoreVersion::Enum)
                .values([ScoreVersion::V1, ScoreVersion::V2])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(RankingType::Enum)
                .values([
                    RankingType::ScoreV1,
                    RankingType::ScoreV2,
                    RankingType::PPV1,
                    RankingType::PPV2,
                ])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(ChannelType::Enum)
                .values([
                    ChannelType::Private,
                    ChannelType::Public,
                    ChannelType::Group,
                    ChannelType::Multiplayer,
                    ChannelType::Spectaor,
                ])
                .to_owned(),
            extension::postgres::Type::create()
                .as_enum(ChannelHandleType::Enum)
                .values([
                    ChannelHandleType::Join,
                    ChannelHandleType::SendMessage,
                    ChannelHandleType::KickUser,
                    ChannelHandleType::MuteUser,
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

        if manager.get_database_backend() != DbBackend::Sqlite {
            for stmt in create_foreign_key_stmts {
                manager.create_foreign_key(stmt).await?;
            }
        }

        for stmt in create_index_stmts {
            manager.create_index(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_table_stmts = vec![
            users::drop(),
            privileges::drop(),
            user_privileges::drop(),
            bancho_client_hardware_records::drop(),
            favourite_beatmaps::drop(),
            followers::drop(),
            user_settings::drop(),
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
            score_pp_standard::drop(),
            score_pp_taiko::drop(),
            score_pp_fruits::drop(),
            score_pp_mania::drop(),
            score_pp_standard_relax::drop(),
            score_pp_standard_autopilot::drop(),
            score_pp_taiko_relax::drop(),
            score_pp_fruits_relax::drop(),
            user_stats_standard::drop(),
            user_stats_standard_score_v2::drop(),
            user_stats_taiko::drop(),
            user_stats_fruits::drop(),
            user_stats_mania::drop(),
            user_stats_standard_relax::drop(),
            user_stats_standard_autopilot::drop(),
            user_stats_taiko_relax::drop(),
            user_stats_fruits_relax::drop(),
            user_pp_standard::drop(),
            user_pp_taiko::drop(),
            user_pp_fruits::drop(),
            user_pp_mania::drop(),
            user_pp_standard_relax::drop(),
            user_pp_standard_autopilot::drop(),
            user_pp_taiko_relax::drop(),
            user_pp_fruits_relax::drop(),
            leaderboard_standard::drop(),
            leaderboard_taiko::drop(),
            leaderboard_fruits::drop(),
            leaderboard_mania::drop(),
            leaderboard_standard_relax::drop(),
            leaderboard_standard_autopilot::drop(),
            leaderboard_taiko_relax::drop(),
            leaderboard_fruits_relax::drop(),
            channels::drop(),
            channel_users::drop(),
            channel_privileges::drop(),
            chat_messages::drop(),
        ];

        let drop_foreign_key_stmts = vec![
            user_privileges::drop_foreign_keys(),
            bancho_client_hardware_records::drop_foreign_keys(),
            favourite_beatmaps::drop_foreign_keys(),
            followers::drop_foreign_keys(),
            user_settings::drop_foreign_keys(),
            beatmap_ratings::drop_foreign_keys(),
            scores_standard::drop_foreign_keys(),
            scores_taiko::drop_foreign_keys(),
            scores_fruits::drop_foreign_keys(),
            scores_mania::drop_foreign_keys(),
            scores_standard_relax::drop_foreign_keys(),
            scores_standard_autopilot::drop_foreign_keys(),
            scores_taiko_relax::drop_foreign_keys(),
            scores_fruits_relax::drop_foreign_keys(),
            score_pp_standard::drop_foreign_keys(),
            user_stats_standard_score_v2::drop_foreign_keys(),
            score_pp_taiko::drop_foreign_keys(),
            score_pp_fruits::drop_foreign_keys(),
            score_pp_mania::drop_foreign_keys(),
            score_pp_standard_relax::drop_foreign_keys(),
            score_pp_standard_autopilot::drop_foreign_keys(),
            score_pp_taiko_relax::drop_foreign_keys(),
            score_pp_fruits_relax::drop_foreign_keys(),
            user_stats_standard::drop_foreign_keys(),
            user_stats_taiko::drop_foreign_keys(),
            user_stats_fruits::drop_foreign_keys(),
            user_stats_mania::drop_foreign_keys(),
            user_stats_standard_relax::drop_foreign_keys(),
            user_stats_standard_autopilot::drop_foreign_keys(),
            user_stats_taiko_relax::drop_foreign_keys(),
            user_stats_fruits_relax::drop_foreign_keys(),
            user_pp_standard::drop_foreign_keys(),
            user_pp_taiko::drop_foreign_keys(),
            user_pp_fruits::drop_foreign_keys(),
            user_pp_mania::drop_foreign_keys(),
            user_pp_standard_relax::drop_foreign_keys(),
            user_pp_standard_autopilot::drop_foreign_keys(),
            user_pp_taiko_relax::drop_foreign_keys(),
            user_pp_fruits_relax::drop_foreign_keys(),
            leaderboard_standard::drop_foreign_keys(),
            leaderboard_taiko::drop_foreign_keys(),
            leaderboard_fruits::drop_foreign_keys(),
            leaderboard_mania::drop_foreign_keys(),
            leaderboard_standard_relax::drop_foreign_keys(),
            leaderboard_standard_autopilot::drop_foreign_keys(),
            leaderboard_taiko_relax::drop_foreign_keys(),
            leaderboard_fruits_relax::drop_foreign_keys(),
            channel_users::drop_foreign_keys(),
            channel_privileges::drop_foreign_keys(),
            chat_messages::drop_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let drop_index_stmts = vec![
            users::drop_indexes(),
            privileges::drop_indexes(),
            user_privileges::drop_indexes(),
            favourite_beatmaps::drop_indexes(),
            followers::drop_indexes(),
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
            score_pp_standard::drop_indexes(),
            score_pp_taiko::drop_indexes(),
            score_pp_fruits::drop_indexes(),
            score_pp_mania::drop_indexes(),
            score_pp_standard_relax::drop_indexes(),
            score_pp_standard_autopilot::drop_indexes(),
            score_pp_taiko_relax::drop_indexes(),
            score_pp_fruits_relax::drop_indexes(),
            user_pp_standard::drop_indexes(),
            user_pp_taiko::drop_indexes(),
            user_pp_fruits::drop_indexes(),
            user_pp_mania::drop_indexes(),
            user_pp_standard_relax::drop_indexes(),
            user_pp_standard_autopilot::drop_indexes(),
            user_pp_taiko_relax::drop_indexes(),
            user_pp_fruits_relax::drop_indexes(),
            leaderboard_standard::drop_indexes(),
            leaderboard_taiko::drop_indexes(),
            leaderboard_fruits::drop_indexes(),
            leaderboard_mania::drop_indexes(),
            leaderboard_standard_relax::drop_indexes(),
            leaderboard_standard_autopilot::drop_indexes(),
            leaderboard_taiko_relax::drop_indexes(),
            leaderboard_fruits_relax::drop_indexes(),
            channels::drop_indexes(),
            channel_users::drop_indexes(),
            channel_privileges::drop_indexes(),
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
            extension::postgres::Type::drop().name(PPVersion::Enum).to_owned(),
            extension::postgres::Type::drop()
                .name(ScoreVersion::Enum)
                .to_owned(),
            extension::postgres::Type::drop()
                .name(RankingType::Enum)
                .to_owned(),
            extension::postgres::Type::drop()
                .name(ChannelType::Enum)
                .to_owned(),
            extension::postgres::Type::drop()
                .name(ChannelHandleType::Enum)
                .to_owned(),
        ];

        if manager.get_database_backend() != DbBackend::Sqlite {
            for stmt in drop_foreign_key_stmts {
                manager.drop_foreign_key(stmt).await?;
            }
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
            .col(ColumnDef::new(Users::Country).string().string_len(8).null())
            .col(
                ColumnDef::new(Users::CreatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(
                ColumnDef::new(Users::UpdatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
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

pub mod privileges {
    use sea_orm_migration::prelude::*;

    const INDEX_NAME: &str = "IDX_privileges_name";
    const INDEX_PRIORITY: &str = "IDX_privileges_priority";

    #[derive(Iden)]
    pub enum Privileges {
        Table,
        Id,
        Name,
        Description,
        Priority,
        CreatorId,
        CreatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(Privileges::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Privileges::Id)
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(Privileges::Name)
                    .string()
                    .unique_key()
                    .not_null(),
            )
            .col(ColumnDef::new(Privileges::Description).string().null())
            .col(
                ColumnDef::new(Privileges::Priority)
                    .small_integer()
                    .not_null()
                    .default(1000),
            )
            .col(ColumnDef::new(Privileges::CreatorId).integer().null())
            .col(
                ColumnDef::new(Privileges::CreatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(Privileges::Table).to_owned()
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![
            sea_query::Index::create()
                .name(INDEX_NAME)
                .table(Privileges::Table)
                .col(Privileges::Name)
                .unique()
                .to_owned(),
            sea_query::Index::create()
                .name(INDEX_PRIORITY)
                .table(Privileges::Table)
                .col(Privileges::Priority)
                .unique()
                .to_owned(),
        ]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![
            sea_query::Index::drop()
                .table(Privileges::Table)
                .name(INDEX_NAME)
                .to_owned(),
            sea_query::Index::drop()
                .table(Privileges::Table)
                .name(INDEX_PRIORITY)
                .to_owned(),
        ]
    }
}

pub mod user_privileges {
    use sea_orm_migration::prelude::*;

    use super::{privileges::Privileges, users::Users};

    const FOREIGN_KEY_USER_ID: &str = "FK_user_priv_user_id";
    const FOREIGN_KEY_PRIV_ID: &str = "FK_user_priv_priv_id";
    const FOREIGN_KEY_GRANTOR_ID: &str = "FK_user_priv_grantor_id";

    const INDEX_PRIV_ID: &str = "IDX_user_priv_priv_id";

    #[derive(Iden)]
    pub enum UserPrivileges {
        Table,
        UserId,
        PrivilegeId,
        GrantorId,
        CreatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(UserPrivileges::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(UserPrivileges::UserId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UserPrivileges::PrivilegeId)
                    .big_integer()
                    .not_null(),
            )
            .col(ColumnDef::new(UserPrivileges::GrantorId).integer().not_null())
            .col(
                ColumnDef::new(UserPrivileges::CreatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(UserPrivileges::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_USER_ID)
                .from(UserPrivileges::Table, UserPrivileges::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_PRIV_ID)
                .from(UserPrivileges::Table, UserPrivileges::PrivilegeId)
                .to(Privileges::Table, Privileges::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_GRANTOR_ID)
                .from(UserPrivileges::Table, UserPrivileges::GrantorId)
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
                .table(UserPrivileges::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_PRIV_ID)
                .table(UserPrivileges::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_GRANTOR_ID)
                .table(UserPrivileges::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_PRIV_ID)
            .table(UserPrivileges::Table)
            .col(UserPrivileges::PrivilegeId)
            .unique()
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(UserPrivileges::Table)
            .name(INDEX_PRIV_ID)
            .to_owned()]
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
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::UpdatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
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
        BeatmapsetId,
        Comment,
        CreatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(FavouriteBeatmaps::Table)
            .if_not_exists()
            .col(ColumnDef::new(FavouriteBeatmaps::UserId).integer().not_null())
            .col(
                ColumnDef::new(FavouriteBeatmaps::BeatmapsetId)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(FavouriteBeatmaps::Comment)
                    .string()
                    .string_len(15)
                    .null(),
            )
            .col(
                ColumnDef::new(FavouriteBeatmaps::CreatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(FavouriteBeatmaps::UserId)
                    .col(FavouriteBeatmaps::BeatmapsetId),
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

pub mod followers {
    use sea_orm_migration::prelude::*;

    use super::users::Users;

    const FOREIGN_KEY_USER_ID: &str = "FK_followers_user_id";
    const FOREIGN_KEY_FOLLOW_ID: &str = "FK_followers_follow_id";
    const INDEX_USER_ID: &str = "IDX_followers_user_id";

    #[derive(Iden)]
    pub enum Followers {
        Table,
        UserId,
        FollowId,
        Remark,
        CreatedAt,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(Followers::Table)
            .if_not_exists()
            .col(ColumnDef::new(Followers::UserId).integer().not_null())
            .col(ColumnDef::new(Followers::FollowId).integer().not_null())
            .col(
                ColumnDef::new(Followers::Remark)
                    .string()
                    .string_len(16)
                    .null(),
            )
            .col(
                ColumnDef::new(Followers::CreatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(Followers::UserId)
                    .col(Followers::FollowId),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(Followers::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_USER_ID)
                .from(Followers::Table, Followers::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_FOLLOW_ID)
                .from(Followers::Table, Followers::FollowId)
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
                .table(Followers::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_FOLLOW_ID)
                .table(Followers::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_USER_ID)
            .table(Followers::Table)
            .col(Followers::UserId)
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(Followers::Table)
            .name(INDEX_USER_ID)
            .to_owned()]
    }
}

pub mod user_settings {
    use sea_orm_migration::prelude::*;

    use super::{users::Users, RankingType};

    const FOREIGN_KEY_USER_ID: &str = "FK_user_settings_user_id";

    #[derive(Iden)]
    pub enum UserSettings {
        Table,
        UserId,
        DisplayUnicodeName,
        ScoreboardRankingType,
        InvisibleOnline,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(UserSettings::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(UserSettings::UserId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UserSettings::DisplayUnicodeName)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(UserSettings::ScoreboardRankingType)
                    .enumeration(
                        RankingType::Enum,
                        [
                            RankingType::ScoreV1,
                            RankingType::ScoreV2,
                            RankingType::PPV1,
                            RankingType::PPV2,
                        ],
                    )
                    .not_null()
                    .default(RankingType::ScoreV1.to_string()),
            )
            .col(
                ColumnDef::new(UserSettings::InvisibleOnline)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(UserSettings::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![sea_query::ForeignKey::create()
            .name(FOREIGN_KEY_USER_ID)
            .from(UserSettings::Table, UserSettings::UserId)
            .to(Users::Table, Users::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .to_owned()]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![sea_query::ForeignKey::drop()
            .name(FOREIGN_KEY_USER_ID)
            .table(UserSettings::Table)
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
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::UploadTime)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(
                ColumnDef::new(Beatmaps::ApprovedTime)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .null(),
            )
            .col(
                ColumnDef::new(Beatmaps::UpdatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
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
                    .tiny_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BeatmapRatings::UpdatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
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

macro_rules! define_user_mode_scores {
    ($table_name: ident, $iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::{users::Users, ScoreGrade, ScoreStatus, ScoreVersion};

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
                ScoreVersion,
                Score,
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
                    .col(
                        ColumnDef::new($iden::ScoreVersion)
                            .enumeration(
                                ScoreVersion::Enum,
                                [ScoreVersion::V1, ScoreVersion::V2],
                            )
                            .not_null()
                            .default(ScoreVersion::V1.to_string()),
                    )
                    .col(ColumnDef::new($iden::Score).integer().not_null())
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
                            .default(Expr::current_timestamp())
                            .null(),
                    )
                    .col(
                        ColumnDef::new($iden::CreateAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new($iden::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
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

define_user_mode_scores!(scores_standard, ScoresStandard);
define_user_mode_scores!(scores_taiko, ScoresTaiko);
define_user_mode_scores!(scores_fruits, ScoresFruits);
define_user_mode_scores!(scores_mania, ScoresMania);
define_user_mode_scores!(scores_standard_relax, ScoresStandardRelax);
define_user_mode_scores!(scores_standard_autopilot, ScoresStandardAutopilot);
define_user_mode_scores!(scores_taiko_relax, ScoresTaikoRelax);
define_user_mode_scores!(scores_fruits_relax, ScoresFruitsRelax);

macro_rules! define_score_mode_pp {
    ($table_name: ident, $iden: ident, $relate_table: ident :: $relate_iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::{$relate_table::$relate_iden, PPVersion};

            const FOREIGN_KEY_SCORE_ID: &str =
                concat!("FK_", stringify!($table_name), "_score_id");
            const INDEX_PP_WITH_VERSION: &str =
                concat!("IDX_", stringify!($table_name), "_pp");

            #[derive(Iden)]
            pub enum $iden {
                Table,
                ScoreId,
                PPVersion,
                PP,
                RawPP,
            }

            pub fn create() -> TableCreateStatement {
                Table::create()
                    .table($iden::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new($iden::ScoreId).big_integer().not_null(),
                    )
                    .col(
                        ColumnDef::new($iden::PPVersion)
                            .enumeration(
                                PPVersion::Enum,
                                [PPVersion::V1, PPVersion::V2],
                            )
                            .not_null()
                            .default(PPVersion::V2.to_string()),
                    )
                    .col(
                        ColumnDef::new($iden::PP)
                            .decimal()
                            .decimal_len(16, 2)
                            .not_null()
                            .default(0.0),
                    )
                    .col(ColumnDef::new($iden::RawPP).json().null())
                    .primary_key(
                        sea_query::Index::create()
                            .col($iden::ScoreId)
                            .col($iden::PPVersion),
                    )
                    .to_owned()
            }

            pub fn drop() -> TableDropStatement {
                Table::drop().table($iden::Table).to_owned()
            }

            pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
                vec![sea_query::ForeignKey::create()
                    .name(FOREIGN_KEY_SCORE_ID)
                    .from($iden::Table, $iden::ScoreId)
                    .to($relate_iden::Table, $relate_iden::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned()]
            }

            pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
                vec![sea_query::ForeignKey::drop()
                    .name(FOREIGN_KEY_SCORE_ID)
                    .table($iden::Table)
                    .to_owned()]
            }

            pub fn create_indexes() -> Vec<IndexCreateStatement> {
                vec![sea_query::Index::create()
                    .name(INDEX_PP_WITH_VERSION)
                    .table($iden::Table)
                    .col($iden::PPVersion)
                    .col($iden::PP)
                    .to_owned()]
            }

            pub fn drop_indexes() -> Vec<IndexDropStatement> {
                vec![sea_query::Index::drop()
                    .table($iden::Table)
                    .name(INDEX_PP_WITH_VERSION)
                    .to_owned()]
            }
        }
    };
}

define_score_mode_pp!(
    score_pp_standard,
    ScorePPStandard,
    scores_standard::ScoresStandard
);
define_score_mode_pp!(score_pp_taiko, ScorePPTaiko, scores_taiko::ScoresTaiko);
define_score_mode_pp!(
    score_pp_fruits,
    ScorePPFruits,
    scores_fruits::ScoresFruits
);
define_score_mode_pp!(score_pp_mania, ScorePPMania, scores_mania::ScoresMania);
define_score_mode_pp!(
    score_pp_standard_relax,
    ScorePPStandardRelax,
    scores_standard_relax::ScoresStandardRelax
);
define_score_mode_pp!(
    score_pp_standard_autopilot,
    ScorePPStandardAutopilot,
    scores_standard_autopilot::ScoresStandardAutopilot
);
define_score_mode_pp!(
    score_pp_taiko_relax,
    ScorePPTaikoRelax,
    scores_taiko_relax::ScoresTaikoRelax
);
define_score_mode_pp!(
    score_pp_fruits_relax,
    ScorePPFruitsRelax,
    scores_fruits_relax::ScoresFruitsRelax
);

macro_rules! define_user_mode_stats {
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
                            .default(Expr::current_timestamp())
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

define_user_mode_stats!(user_stats_standard, UserStatsStandard);
define_user_mode_stats!(user_stats_standard_score_v2, UserStatsStandardScoreV2);
define_user_mode_stats!(user_stats_taiko, UserStatsTaiko);
define_user_mode_stats!(user_stats_fruits, UserStatsFruits);
define_user_mode_stats!(user_stats_mania, UserStatsMania);
define_user_mode_stats!(user_stats_standard_relax, UserStatsStandardRelax);
define_user_mode_stats!(
    user_stats_standard_autopilot,
    UserStatsStandardAutopilot
);
define_user_mode_stats!(user_stats_taiko_relax, UserStatsTaikoRelax);
define_user_mode_stats!(user_stats_fruits_relax, UserStatsFruitsRelax);

macro_rules! define_user_mode_pp {
    ($table_name: ident, $iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::{users::Users, PPVersion};

            const FOREIGN_KEY_USER_ID: &str =
                concat!("FK_", stringify!($table_name), "_user_id");
            const INDEX_PP_WITH_VERSION: &str =
                concat!("IDX_", stringify!($table_name), "_pp");

            #[derive(Iden)]
            pub enum $iden {
                Table,
                UserId,
                PPVersion,
                PP,
                RawPP,
            }

            pub fn create() -> TableCreateStatement {
                Table::create()
                    .table($iden::Table)
                    .if_not_exists()
                    .col(ColumnDef::new($iden::UserId).integer().not_null())
                    .col(
                        ColumnDef::new($iden::PPVersion)
                            .enumeration(
                                PPVersion::Enum,
                                [PPVersion::V1, PPVersion::V2],
                            )
                            .not_null()
                            .default(PPVersion::V2.to_string()),
                    )
                    .col(
                        ColumnDef::new($iden::PP)
                            .decimal()
                            .decimal_len(16, 2)
                            .not_null()
                            .default(0.0),
                    )
                    .col(ColumnDef::new($iden::RawPP).json().null())
                    .primary_key(
                        sea_query::Index::create()
                            .col($iden::UserId)
                            .col($iden::PPVersion),
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
                vec![sea_query::Index::create()
                    .name(INDEX_PP_WITH_VERSION)
                    .table($iden::Table)
                    .col($iden::PPVersion)
                    .col($iden::PP)
                    .to_owned()]
            }

            pub fn drop_indexes() -> Vec<IndexDropStatement> {
                vec![sea_query::Index::drop()
                    .table($iden::Table)
                    .name(INDEX_PP_WITH_VERSION)
                    .to_owned()]
            }
        }
    };
}

define_user_mode_pp!(user_pp_standard, UserPPStandard);
define_user_mode_pp!(user_pp_taiko, UserPPTaiko);
define_user_mode_pp!(user_pp_fruits, UserPPFruits);
define_user_mode_pp!(user_pp_mania, UserPPMania);
define_user_mode_pp!(user_pp_standard_relax, UserPPStandardRelax);
define_user_mode_pp!(user_pp_standard_autopilot, UserPPStandardAutopilot);
define_user_mode_pp!(user_pp_taiko_relax, UserPPTaikoRelax);
define_user_mode_pp!(user_pp_fruits_relax, UserPPFruitsRelax);

macro_rules! define_beatmap_mode_leaderboard {
    ($table_name: ident, $iden: ident, $relate_table: ident :: $relate_iden: ident) => {
        pub mod $table_name {
            use sea_orm_migration::prelude::*;

            use super::{
                beatmaps::Beatmaps, users::Users, $relate_table::$relate_iden,
                RankingType,
            };

            const FOREIGN_KEY_BEATMAP_ID: &str =
                concat!("FK_", stringify!($table_name), "_beatmap_id");
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
                RankingType,
                UserId,
                ScoreId,
            }

            pub fn create() -> TableCreateStatement {
                Table::create()
                    .table($iden::Table)
                    .if_not_exists()
                    .col(ColumnDef::new($iden::BeatmapId).integer().not_null())
                    .col(
                        ColumnDef::new($iden::RankingType)
                            .enumeration(
                                RankingType::Enum,
                                [
                                    RankingType::ScoreV1,
                                    RankingType::ScoreV2,
                                    RankingType::PPV1,
                                    RankingType::PPV2,
                                ],
                            )
                            .not_null()
                            .default(RankingType::ScoreV1.to_string()),
                    )
                    .col(ColumnDef::new($iden::UserId).integer().not_null())
                    .col(
                        ColumnDef::new($iden::ScoreId).big_integer().not_null(),
                    )
                    .primary_key(
                        sea_query::Index::create()
                            .col($iden::BeatmapId)
                            .col($iden::RankingType),
                    )
                    .to_owned()
            }

            pub fn drop() -> TableDropStatement {
                Table::drop().table($iden::Table).to_owned()
            }

            pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
                vec![
                    sea_query::ForeignKey::create()
                        .name(FOREIGN_KEY_BEATMAP_ID)
                        .from($iden::Table, $iden::BeatmapId)
                        .to(Beatmaps::Table, Beatmaps::Bid)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        .to_owned(),
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
                        .name(FOREIGN_KEY_BEATMAP_ID)
                        .table($iden::Table)
                        .to_owned(),
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

define_beatmap_mode_leaderboard!(
    leaderboard_standard,
    LeaderboardStandard,
    scores_standard::ScoresStandard
);
define_beatmap_mode_leaderboard!(
    leaderboard_taiko,
    LeaderboardTaiko,
    scores_taiko::ScoresTaiko
);
define_beatmap_mode_leaderboard!(
    leaderboard_fruits,
    LeaderboardFruits,
    scores_fruits::ScoresFruits
);
define_beatmap_mode_leaderboard!(
    leaderboard_mania,
    LeaderboardMania,
    scores_mania::ScoresMania
);
define_beatmap_mode_leaderboard!(
    leaderboard_standard_relax,
    LeaderboardStandardRelax,
    scores_standard_relax::ScoresStandardRelax
);
define_beatmap_mode_leaderboard!(
    leaderboard_standard_autopilot,
    LeaderboardStandardAutopilot,
    scores_standard_autopilot::ScoresStandardAutopilot
);
define_beatmap_mode_leaderboard!(
    leaderboard_taiko_relax,
    LeaderboardTaikoRelax,
    scores_taiko_relax::ScoresTaikoRelax
);
define_beatmap_mode_leaderboard!(
    leaderboard_fruits_relax,
    LeaderboardFruitsRelax,
    scores_fruits_relax::ScoresFruitsRelax
);

pub mod channels {
    use sea_orm_migration::prelude::*;

    use super::ChannelType;

    const INDEX_CHANNEL_NAME: &str = "IDX_channel_name";

    #[derive(Iden)]
    pub enum Channels {
        Table,
        Id,
        ChannelType,
        Name,
        Description,
        Icon,
        AutoJoin,
        CreatorId,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(Channels::Table)
            .if_not_exists()
            .col(ColumnDef::new(Channels::Id).big_integer().not_null())
            .col(
                ColumnDef::new(Channels::ChannelType)
                    .enumeration(
                        ChannelType::Enum,
                        [
                            ChannelType::Private,
                            ChannelType::Public,
                            ChannelType::Group,
                            ChannelType::Multiplayer,
                            ChannelType::Spectaor,
                        ],
                    )
                    .not_null(),
            )
            .col(ColumnDef::new(Channels::Name).string().unique_key().null())
            .col(ColumnDef::new(Channels::Description).string().null())
            .col(ColumnDef::new(Channels::Icon).string().null())
            .col(
                ColumnDef::new(Channels::AutoJoin)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(Channels::CreatorId).big_integer().null())
            .primary_key(sea_query::Index::create().col(Channels::Id))
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(Channels::Table).to_owned()
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_CHANNEL_NAME)
            .table(Channels::Table)
            .col(Channels::Name)
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(Channels::Table)
            .name(INDEX_CHANNEL_NAME)
            .to_owned()]
    }
}

pub mod channel_users {
    use sea_orm_migration::prelude::*;

    use super::{channels::Channels, users::Users};

    const FOREIGN_KEY_CHANNEL_ID: &str = "FK_channel_users_channel_id";
    const FOREIGN_KEY_USER_ID: &str = "FK_channel_users_user_id";
    const INDEX_USER_ID: &str = "IDX_channel_users_user_id";

    #[derive(Iden)]
    pub enum ChannelUsers {
        Table,
        ChannelId,
        UserId,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(ChannelUsers::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ChannelUsers::ChannelId)
                    .big_integer()
                    .not_null(),
            )
            .col(ColumnDef::new(ChannelUsers::UserId).integer().not_null())
            .primary_key(
                sea_query::Index::create()
                    .col(ChannelUsers::ChannelId)
                    .col(ChannelUsers::UserId),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(ChannelUsers::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_CHANNEL_ID)
                .from(ChannelUsers::Table, ChannelUsers::ChannelId)
                .to(Channels::Table, Channels::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_USER_ID)
                .from(ChannelUsers::Table, ChannelUsers::UserId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
        ]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_CHANNEL_ID)
                .table(ChannelUsers::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_USER_ID)
                .table(ChannelUsers::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_USER_ID)
            .table(ChannelUsers::Table)
            .col(ChannelUsers::UserId)
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(ChannelUsers::Table)
            .name(INDEX_USER_ID)
            .to_owned()]
    }
}

pub mod channel_privileges {
    use sea_orm_migration::prelude::*;

    use super::{
        channels::Channels, privileges::Privileges, ChannelHandleType,
    };

    const FOREIGN_KEY_CHANNEL_ID: &str = "FK_channel_priv_channel_id";
    const FOREIGN_KEY_PRIV_ID: &str = "FK_channel_priv_priv_id";

    const INDEX_PRIV_ID: &str = "IDX_channel_priv_priv_id";

    #[derive(Iden)]
    pub enum ChannelPrivileges {
        Table,
        ChannelId,
        Handle,
        RequiredPrivilegeId,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(ChannelPrivileges::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ChannelPrivileges::ChannelId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ChannelPrivileges::Handle)
                    .enumeration(
                        ChannelHandleType::Enum,
                        [
                            ChannelHandleType::Join,
                            ChannelHandleType::SendMessage,
                            ChannelHandleType::KickUser,
                            ChannelHandleType::MuteUser,
                        ],
                    )
                    .not_null(),
            )
            .col(
                ColumnDef::new(ChannelPrivileges::RequiredPrivilegeId)
                    .big_integer()
                    .not_null(),
            )
            .primary_key(
                sea_query::Index::create()
                    .col(ChannelPrivileges::ChannelId)
                    .col(ChannelPrivileges::Handle),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(ChannelPrivileges::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_CHANNEL_ID)
                .from(ChannelPrivileges::Table, ChannelPrivileges::ChannelId)
                .to(Channels::Table, Channels::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_PRIV_ID)
                .from(
                    ChannelPrivileges::Table,
                    ChannelPrivileges::RequiredPrivilegeId,
                )
                .to(Privileges::Table, Privileges::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
        ]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_CHANNEL_ID)
                .table(ChannelPrivileges::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_PRIV_ID)
                .table(ChannelPrivileges::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_PRIV_ID)
            .table(ChannelPrivileges::Table)
            .col(ChannelPrivileges::RequiredPrivilegeId)
            .unique()
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(ChannelPrivileges::Table)
            .name(INDEX_PRIV_ID)
            .to_owned()]
    }
}

pub mod chat_messages {
    use sea_orm_migration::prelude::*;

    use super::{channels::Channels, users::Users};

    const FOREIGN_KEY_CHANNEL_ID: &str = "FK_chat_msg_channel_id";
    const FOREIGN_KEY_USER_ID: &str = "FK_chat_msg_user_id";
    const INDEX_CHANNEL_ID: &str = "IDX_chat_msg_channel_id";

    #[derive(Iden)]
    pub enum ChatMessages {
        Table,
        Id,
        SenderId,
        ChannelId,
        Timestamp,
        ContentString,
        ContentHtml,
        IsAction,
    }

    pub fn create() -> TableCreateStatement {
        Table::create()
            .table(ChatMessages::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ChatMessages::Id)
                    .big_integer()
                    .primary_key()
                    .auto_increment()
                    .not_null(),
            )
            .col(ColumnDef::new(ChatMessages::SenderId).integer().not_null())
            .col(
                ColumnDef::new(ChatMessages::ChannelId)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(ChatMessages::Timestamp)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(ColumnDef::new(ChatMessages::ContentString).text().not_null())
            .col(ColumnDef::new(ChatMessages::ContentHtml).text().null())
            .col(
                ColumnDef::new(ChatMessages::IsAction)
                    .boolean()
                    .default(false)
                    .not_null(),
            )
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(ChatMessages::Table).to_owned()
    }

    pub fn create_foreign_keys() -> Vec<ForeignKeyCreateStatement> {
        vec![
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_CHANNEL_ID)
                .from(ChatMessages::Table, ChatMessages::ChannelId)
                .to(Channels::Table, Channels::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
            sea_query::ForeignKey::create()
                .name(FOREIGN_KEY_USER_ID)
                .from(ChatMessages::Table, ChatMessages::SenderId)
                .to(Users::Table, Users::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned(),
        ]
    }

    pub fn drop_foreign_keys() -> Vec<ForeignKeyDropStatement> {
        vec![
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_CHANNEL_ID)
                .table(ChatMessages::Table)
                .to_owned(),
            sea_query::ForeignKey::drop()
                .name(FOREIGN_KEY_USER_ID)
                .table(ChatMessages::Table)
                .to_owned(),
        ]
    }

    pub fn create_indexes() -> Vec<IndexCreateStatement> {
        vec![sea_query::Index::create()
            .name(INDEX_CHANNEL_ID)
            .table(ChatMessages::Table)
            .col(ChatMessages::ChannelId)
            .unique()
            .to_owned()]
    }

    pub fn drop_indexes() -> Vec<IndexDropStatement> {
        vec![sea_query::Index::drop()
            .table(ChatMessages::Table)
            .name(INDEX_CHANNEL_ID)
            .to_owned()]
    }
}
