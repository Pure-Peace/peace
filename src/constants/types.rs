#![allow(dead_code)]
use async_std::sync::RwLock;
use std::sync::Arc;

pub type TestType = Arc<RwLock<i32>>;
