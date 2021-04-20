mod address;
mod base;
mod data;
mod status;
mod play_mods;
mod player;
mod settings;
mod stats;
mod game_status;

mod depends {
    pub use super::{stats::Stats, game_status::GameStatus, PlayerBase};

    pub use crate::{
        constants::{Action, ClientInfo, GameMode, PlayMod, PresenceFilter},
        objects::Channel,
        types::{Location, PacketData, TokenString},
    };
    pub use crate::{
        constants::{BanchoPrivileges, Privileges},
        database::Database,
    };

    pub use actix_web::web::Data;
    pub use async_std::sync::{Mutex, RwLock};
    pub use chrono::prelude::{DateTime, Local};
    pub use hashbrown::{HashMap, HashSet};
    pub use num_traits::FromPrimitive;
    pub use queue::Queue;
    pub use serde::Deserialize;
    pub use std::time::Instant;
    pub use strum::IntoEnumIterator;
    pub use uuid::Uuid;
}

pub use address::PlayerAddress;
pub use base::PlayerBase;
pub use data::PlayerData;
pub use status::PlayerStatus;
pub use play_mods::PlayMods;
pub use player::Player;
pub use settings::PlayerSettings;
