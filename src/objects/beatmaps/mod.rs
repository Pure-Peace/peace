mod beatmap;
mod from_api;
mod info;

pub use beatmap::*;
pub use from_api::*;
pub use info::*;

#[derive(Debug)]
pub enum GetBeatmapMethod {
    Md5,
    Bid,
    Sid,
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
