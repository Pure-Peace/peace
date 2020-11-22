#![allow(dead_code)]
use async_std::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerBase {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
}

impl Player {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    pub async fn from_base(base: PlayerBase) -> Self {
        Player {
            id: base.id,
            name: base.name,
            privileges: base.privileges,
            country: base.country,
        }
    }
}
