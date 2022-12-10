use crate::define_db_config;

pub mod entity;

#[cfg(feature = "migration")]
pub mod migration;

define_db_config!(PeaceDbConfig, peace);
