use peace_cfg::define_db_config;

pub mod entity;
pub mod migration;

define_db_config!(config_name: PeaceDbConfig, command_prefix: peace);
