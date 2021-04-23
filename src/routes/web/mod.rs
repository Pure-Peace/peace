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

    pub use peace_database::Database;
    pub use peace_settings::bancho::BanchoConfig;

    pub use actix_multipart::Multipart;
    pub use serde::Deserialize;
    pub use serde_json::json;

    use crate::objects::{Bancho, Caches};
    pub use crate::{
        handlers::bancho,
        objects::{Player, PlayerData, PlayerSessions},
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
