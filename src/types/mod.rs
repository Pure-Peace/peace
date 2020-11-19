#![allow(dead_code)]
use async_std::sync::RwLock;

use crate::objects::Player;

pub type TestType = RwLock<i32>;
pub type TokenString = String;
pub type UserId = i32;

pub type PlayerHandler = fn(player: &mut Player);
pub type PlayerSessionMap = RwLock<hashbrown::HashMap<TokenString, Player>>;
pub type PlayerSessionMapData = hashbrown::HashMap<TokenString, Player>;
pub type PlayerIdSessionMap = RwLock<hashbrown::HashMap<UserId, TokenString>>;

pub type Username = String;
pub type Password = String;
pub type ClientInfo = Vec<String>;
pub type ClientHashes = Vec<String>;
