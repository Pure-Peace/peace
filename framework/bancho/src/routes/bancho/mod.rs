pub mod get;
pub mod post;

mod login;
mod register;

mod depends {
    pub use {
        askama::Template,
        futures::StreamExt,
        maxminddb::Reader,
        memmap::Mmap,
        ntex::{
            util::Bytes,
            web::{post, types::Data, HttpRequest, HttpResponse},
        },
        ntex_multipart::Multipart,
        prometheus::IntCounterVec,
        serde::Deserialize,
        serde_json::json,
        std::sync::{atomic::Ordering, Arc},
        tokio::sync::RwLock,
        what_i_want::*,
    };

    pub use {
        bancho_packets::{LoginFailed, LoginReply, PacketBuilder, PacketId, PacketReader, server_packet},
        peace_database::Database,
        peace_settings::bancho::BanchoConfig,
    };

    pub use crate::{
        handlers::bancho,
        objects::{Bancho, Caches, Player, PlayerData, PlayerSessions},
        renders::BanchoGet,
        types::{Argon2Cache, ChannelList},
    };
}

pub use register::osu_register;
