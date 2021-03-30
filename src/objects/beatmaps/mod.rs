mod cache;
mod from_api;
mod beatmap;

pub use cache::*;
pub use from_api::*;
pub use beatmap::*;

#[derive(Debug)]
pub enum GetBeatmapMethod {
    Md5,
    Bid,
    Sid,
}

impl GetBeatmapMethod {
    #[inline(always)]
    pub fn db_column_name(&self) -> String {
        match self {
            &Self::Md5 => "md5",
            &Self::Bid => "id",
            &Self::Sid => "set_id",
        }
        .to_string()
    }
}

mod depends {
    pub use crate::database::Database;
    pub use crate::objects::{Bancho, Caches, OsuApi};
    pub use crate::utils::{from_str_bool, from_str_optional};

    pub use actix_web::web::Data;
    pub use async_std::sync::RwLock;
    pub use chrono::{DateTime, Local};
    pub use field_names::FieldNames;
    pub use postgres_types::{FromSql, ToSql};
    pub use serde::Deserialize;
    pub use serde_str;
    pub use std::any::Any;
    pub use std::fmt::Display;
    pub use tokio_pg_mapper_derive::PostgresMapper;
}
