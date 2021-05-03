mod address;
mod base;
mod data;
mod game_status;
mod player;
mod settings;
mod stats;
mod status;

mod depends {
    pub use super::{game_status::GameStatus, stats::Stats, PlayerBase};

    pub use crate::{
        objects::Channel,
        types::{Location, PacketData, TokenString},
    };
    pub use actix_web::web::Data;
    pub use async_std::sync::{Mutex, RwLock};
    pub use chrono::prelude::{DateTime, Local};
    pub use hashbrown::{HashMap, HashSet};
    pub use num_traits::FromPrimitive;
    pub use peace_database::Database;
    pub use queue::Queue;
    pub use serde::Deserialize;
    pub use std::time::Instant;
    pub use uuid::Uuid;
}

pub use address::PlayerAddress;
pub use base::PlayerBase;
pub use data::PlayerData;
pub use player::Player;
pub use settings::PlayerSettings;
pub use status::PlayerStatus;
