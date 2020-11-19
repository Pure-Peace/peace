#![allow(dead_code)]
use async_std::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub money: i32,
    pub age: i32,
}

impl Player {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
