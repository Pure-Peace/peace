pub mod get;
pub mod post;

mod depends {
    pub use {
        tokio::sync::RwLock,
        maxminddb::Reader,
        memmap::Mmap,
        ntex::{
            util::Bytes,
            web::{
                post,
                types::{Data, Path},
                HttpRequest, HttpResponse,
            },
        },
        ntex_multipart::Multipart,
        peace_database::Database,
        peace_settings::bancho::BanchoConfig,
        prometheus::IntCounterVec,
        serde::Deserialize,
        serde_json::json,
        std::sync::{atomic::Ordering, Arc},
    };

    pub use crate::{
        handlers::bancho,
        objects::{Bancho, Caches, Player, PlayerData, PlayerSessions},
        renders::BanchoGet,
        types::{Argon2Cache, ChannelList},
    };

    pub struct Context<'a> {
        pub req: HttpRequest,
        pub counter: &'a Data<IntCounterVec>,
        pub bancho: &'a Data<Bancho>,
        pub database: &'a Data<Database>,
        pub geo_db: &'a Data<Option<Reader<Mmap>>>,
        pub caches: &'a Data<Caches>,
    }
}

pub use depends::Context;
