use actix_web::web::{Bytes, Data};
use async_std::sync::RwLock;

use crate::{
    database::Database,
    objects::{Player, PlayerData, PlayerSessions},
    types::ChannelList,
};

pub struct HandlerData<'a> {
    pub player_sessions: &'a Data<RwLock<PlayerSessions>>,
    pub database: &'a Data<Database>,
    pub channel_list: &'a Data<RwLock<ChannelList>>,
    pub token: &'a String,
    pub player_data: PlayerData,
}

#[derive(Debug)]
pub struct ClientPacket {}

impl ClientPacket {
    pub async fn handle<'a>(&self, handler_data: &HandlerData<'a>) {
        println!("{:?} {:?}", self, handler_data.player_data);
    }
}

pub struct PacketReader {}

impl PacketReader {
    pub async fn parse(body: Bytes) -> Vec<ClientPacket> {
        println!("{:?}\n{:?}", body, body.to_vec());
        vec![]
    }
}
