#![allow(dead_code)]
use async_std::sync::RwLock;

use crate::objects::Player;

pub type TestType = RwLock<i32>;
pub type TokenString = String;
pub type PlayerSessions = RwLock<hashbrown::HashMap<TokenString, Player>>;
