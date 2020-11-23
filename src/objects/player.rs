#![allow(dead_code)]
use async_std::sync::RwLock;
use chrono::prelude::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerBase {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
    pub osu_version: String,
    pub utc_offset: i32,
    pub login_time: DateTime<Local>,
    pub last_active_time: DateTime<Local>
}

impl Player {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    pub async fn from_base(base: PlayerBase, osu_version: String, utc_offset: i32) -> Self {
        let now_time = Local::now();
        Player {
            id: base.id,
            name: base.name,
            privileges: base.privileges,
            country: base.country,
            osu_version,
            utc_offset,
            login_time: now_time,
            last_active_time: now_time,
        }
    }

    pub fn update_active(&mut self) {
        self.last_active_time = Local::now();
    }
}
