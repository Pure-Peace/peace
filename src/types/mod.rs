#![allow(dead_code)]
use std::sync::Arc;

use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashMap;

use crate::objects::{BeatmapCache, Channel, Player};

pub type TestType = RwLock<i32>;
pub type TokenString = String;
pub type PacketData = Vec<u8>;

pub type PlayerSessionMap = RwLock<HashMap<TokenString, Arc<RwLock<Player>>>>;
pub type PlayerIdSessionMap = RwLock<HashMap<UserId, Arc<RwLock<Player>>>>;
pub type PlayerNameSessionMap = RwLock<HashMap<Username, Arc<RwLock<Player>>>>;

pub type ChannelName = String;
pub type ChannelList = HashMap<ChannelName, Channel>;

pub type UserId = i32;
pub type Username = String;
pub type Password = String;
pub type Latitude = f32;
pub type Longitude = f32;
pub type Location = (Latitude, Longitude);

pub type Argon2CryptedCipher = String;
pub type Argon2Cache = HashMap<Argon2CryptedCipher, Password>;

pub type BeatmapMd5 = String;
pub type BeatmapsCache = HashMap<BeatmapMd5, BeatmapCache>; // TODO: like player sessions map, it should can query by bid, sid, md5. need refactors (save Arc<BeatmapCache>)

pub type TempTableCache = HashMap<String, DateTime<Local>>;
