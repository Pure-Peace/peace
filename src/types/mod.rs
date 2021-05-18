#![allow(dead_code)]
use std::sync::Arc;

use tokio::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashMap;

use crate::objects::{Channel, Match, Player};

pub type TestType = RwLock<i32>;
pub type TokenString = String;
pub type PacketData = Vec<u8>;

pub type PlayerSessionMap = HashMap<TokenString, Arc<RwLock<Player>>>;
pub type PlayerIdSessionMap = HashMap<UserId, Arc<RwLock<Player>>>;
pub type PlayerNameSessionMap = HashMap<Username, Arc<RwLock<Player>>>;

pub type ChannelName = String;
pub type ChannelList = HashMap<ChannelName, Arc<RwLock<Channel>>>;

pub type MatchId = i64;
pub type MatchName = String;
pub type MatchList = HashMap<MatchId, Arc<RwLock<Match>>>;

pub type UserId = i32;
pub type Username = String;
pub type Password = String;
pub type Latitude = f32;
pub type Longitude = f32;
pub type Location = (Latitude, Longitude);

pub type Argon2CryptedCipher = String;
pub type Argon2Cache = HashMap<Argon2CryptedCipher, Password>;

pub type BeatmapMd5 = String;

pub type TempTableCache = HashMap<String, DateTime<Local>>;
