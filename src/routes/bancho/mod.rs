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
    pub use peace_constants::LoginFailed;
    pub use peace_database::Database;
    pub use prometheus::IntCounterVec;
    pub use std::sync::{atomic::Ordering, Arc};

    pub use actix_multipart::Multipart;
    pub use futures::StreamExt;
    pub use serde::Deserialize;
    pub use serde_json::json;

    pub use peace_packets::{PacketBuilder, PacketReader};

    pub use crate::{
        handlers::bancho,
        objects::{Bancho, Caches, Player, PlayerData, PlayerSessions},
        renders::BanchoGet,
        settings::bancho::BanchoConfig,
        types::{Argon2Cache, ChannelList},
    };
}

pub use register::osu_register;
