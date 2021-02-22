#![allow(dead_code)]
extern crate config;
extern crate serde;

#[macro_use]
extern crate log;

pub mod constants;
pub mod database;
pub mod events;
pub mod handlers;
pub mod objects;
pub mod packets;
pub mod renders;
pub mod routes;
pub mod settings;
pub mod types;
pub mod utils;
