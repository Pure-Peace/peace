pub mod get;
pub mod post;

mod login;
mod register;

mod depends {
    pub use actix_web::web::{Bytes, Data};
    pub use actix_web::{HttpRequest, HttpResponse, Responder};
    pub use async_std::sync::RwLock;
    pub use maxminddb::Reader;
    pub use memmap::Mmap;
    pub use prometheus::IntCounterVec;

    pub use crate::{
        constants::{id, LoginFailed},
        database::Database,
        handlers::bancho,
        objects::{Player, PlayerData, PlayerSessions},
        packets::PacketBuilder,
        types::{Argon2Cache, ChannelList},
    };
}

pub use register::osu_register;
