#![allow(dead_code)]
use actix_web::web::Data;
use async_std::sync::RwLock;
use hashbrown::HashMap;

use crate::objects::{Channel, Player};

pub type TestType = RwLock<i32>;
pub type TokenString = String;
pub type UserId = i32;
pub type PacketData = Vec<u8>;

pub type PlayerHandler = fn(player: &mut Player);
pub type PlayerSessionMap = RwLock<HashMap<TokenString, Player>>;
pub type PlayerSessionMapData = HashMap<TokenString, Player>;
pub type PlayerIdSessionMap = RwLock<HashMap<UserId, TokenString>>;
pub type ChannelList = HashMap<String, Channel>;

pub type Username = String;
pub type Password = String;
pub type Latitude = f32;
pub type Longitude = f32;
pub type Location = (Latitude, Longitude);
