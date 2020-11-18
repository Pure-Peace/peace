#![allow(dead_code)]
use async_std::sync::RwLock;
use dashmap::DashMap;
use std::sync::Arc;

use crate::objects::Player;

pub type TestType = Arc<RwLock<i32>>;
pub type PlayerMap = Arc<DashMap<i32, Arc<Player>>>;
