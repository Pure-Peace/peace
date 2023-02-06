use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

use super::init_tables::users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::insert()
            .into_table(Users::Table)
            .columns([
                Users::Id,
                Users::Name,
                Users::NameSafe,
                Users::Password,
                Users::Email,
            ])
            .values_panic([
                0.into(),
                "system".into(),
                "system".into(),
                "0".into(),
                "system@email.com".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
