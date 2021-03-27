pub mod get;
pub mod post;

mod depends {
    pub use actix_web::{
        post,
        web::{Bytes, Data, Path},
        HttpRequest, HttpResponse, Responder,
    };
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

    use crate::objects::{Bancho, Caches};
    pub use crate::{
        constants::{id, LoginFailed},
        database::Database,
        handlers::bancho,
        objects::{Player, PlayerData, PlayerSessions},
        packets::{PacketBuilder, PacketReader},
        renders::BanchoGet,
        settings::bancho::BanchoConfig,
        types::{Argon2Cache, ChannelList},
    };

    pub struct Context<'a> {
        pub req: &'a HttpRequest,
        pub counter: &'a Data<IntCounterVec>,
        pub bancho: &'a Data<Bancho>,
        pub database: &'a Data<Database>,
        pub geo_db: &'a Data<Option<Reader<Mmap>>>,
        pub global_cache: &'a Data<Caches>,
    }
}

pub use depends::Context;
