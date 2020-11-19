#![allow(dead_code)]
use async_std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub money: i32,
    pub age: i32,
}


impl Player {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}