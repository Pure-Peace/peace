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
        ];

        let create_foreign_key_stmts = vec![
            bancho_client_hardware_records::create_foreign_keys(),
            favourite_beatmaps::create_foreign_keys(),
            friend_relationships::create_foreign_keys(),
            custom_settings::create_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_index_stmts = vec![
            favourite_beatmaps::create_indexes(),
            friend_relationships::create_indexes(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let create_type_stmts = vec![];

        for stmt in create_table_stmts {
            manager.create_table(stmt).await?;
        }

        for stmt in create_foreign_key_stmts {
            manager.create_foreign_key(stmt).await?;
        }

        for stmt in create_index_stmts {
            manager.create_index(stmt).await?;
        }

        if manager.get_database_backend() == DbBackend::Postgres {
            for stmt in create_type_stmts {
                manager.create_type(stmt).await?;
            }
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
        ];

        let drop_foreign_key_stmts = vec![
            bancho_client_hardware_records::drop_foreign_keys(),
            favourite_beatmaps::drop_foreign_keys(),
            friend_relationships::drop_foreign_keys(),
            custom_settings::drop_foreign_keys(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let drop_index_stmts = vec![
            favourite_beatmaps::drop_indexes(),
            friend_relationships::drop_indexes(),
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
                    .string()
                    .string_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::Adapters)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::AdaptersHash)
                    .string()
                    .string_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::UninstallId)
                    .string()
                    .string_len(32)
                    .not_null(),
            )
            .col(
                ColumnDef::new(BanchoClientHardwareRecords::DiskId)
                    .string()
                    .string_len(32)
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
