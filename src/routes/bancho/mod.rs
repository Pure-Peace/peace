pub mod get;
pub mod post;

mod login;
mod register;

mod depends {
    pub use actix_web::{
        post,
        web::{Bytes, Data},
        HttpRequest, HttpResponse, Responder,
    };
    pub use askama::Template;
    pub use async_std::sync::RwLock;
    pub use maxminddb::Reader;
    pub use memmap::Mmap;
    pub use prometheus::IntCounterVec;
    pub use std::sync::{atomic::Ordering, Arc};

    pub use actix_multipart::Multipart;
    pub use futures::StreamExt;
    pub use regex::Regex;
    pub use serde::Deserialize;
    pub use serde_json::json;

    pub use crate::{
        constants::{id, LoginFailed},
        database::Database,
        handlers::bancho,
        objects::{Bancho, Caches, Player, PlayerData, PlayerSessions},
        packets::{PacketBuilder, PacketReader},
        renders::BanchoGet,
        settings::bancho::BanchoConfig,
        types::{Argon2Cache, ChannelList},
    };
}

pub use register::osu_register;
