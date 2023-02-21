use peace_dal::define_db_config;

pub mod entity;
pub mod repository;

pub use repository::*;

#[cfg(feature = "db_peace_migration")]
pub mod migration;

define_db_config!(PeaceDbConfig, peace);
