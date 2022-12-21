use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

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
        ];

        let create_foreign_key_stmts = vec![
            bancho_client_hardware_records::create_foreign_keys(),
            favourite_beatmaps::create_foreign_keys(),
            friend_relationships::create_foreign_keys(),
            custom_settings::create_foreign_keys(),
            beatmap_ratings::create_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_index_stmts = vec![
            favourite_beatmaps::create_indexes(),
            friend_relationships::create_indexes(),
            beatmaps::create_indexes(),
            beatmap_ratings::create_indexes(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_type_stmts = vec![];

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
        ];

        let drop_foreign_key_stmts = vec![
            bancho_client_hardware_records::drop_foreign_keys(),
            favourite_beatmaps::drop_foreign_keys(),
            friend_relationships::drop_foreign_keys(),
            custom_settings::drop_foreign_keys(),
            beatmap_ratings::drop_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let drop_index_stmts = vec![
            favourite_beatmaps::drop_indexes(),
            friend_relationships::drop_indexes(),
            beatmaps::drop_indexes(),
            beatmap_ratings::drop_indexes(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let drop_type_stmts = vec![];

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
        vec![sea_query::Index::drop().name(INDEX_USER_ID).to_owned()]
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
        vec![sea_query::Index::drop().name(INDEX_USER_ID).to_owned()]
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
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(ColumnDef::new(Beatmaps::GameMode).small_integer().not_null())
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
            sea_query::Index::drop().name(INDEX_SID).to_owned(),
            sea_query::Index::drop().name(INDEX_MD5).to_owned(),
            sea_query::Index::drop().name(INDEX_FILE_NAME).to_owned(),
            sea_query::Index::drop().name(INDEX_RANK_STATUS).to_owned(),
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
                    .string()
                    .string_len(32)
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
        vec![sea_query::Index::drop().name(INDEX_MD5).to_owned()]
    }
}
