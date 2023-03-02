use crate::define_db;

pub mod entity;
pub mod migration;

define_db!(db_name: peace);
