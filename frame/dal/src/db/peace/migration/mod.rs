pub mod versions;

use sea_orm_migration::{MigrationTrait, MigratorTrait};

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(versions::m20221210_000001_init_tables::Migration)]
    }
}
