#![allow(unused_variables)]

use actix_web::web::Data;
use async_std::sync::RwLock;

use crate::{
    constants::id,
    database::Database,
    objects::{PlayerData, PlayerSessions},
    packets::{HandlerData, PayloadReader},
    types::ChannelList,
};

impl id {
    pub async fn handle(
        &self,
        request_ip: &String,
        token: &String,
        player_data: &PlayerData,
        player_sessions: &Data<RwLock<PlayerSessions>>,
        database: &Data<Database>,
        channel_list: &Data<RwLock<ChannelList>>,
        payload: Option<&[u8]>,
    ) {
        match payload {
            None => {
                match self {
                    id::OSU_PING => {}
                    id::OSU_LOGOUT => {
                        player_sessions
                            .write()
                            .await
                            .logout(token, Some(channel_list))
                            .await;
                    }
                    _ => {
                        warn!(
                            "Unhandled packet (Non-payload): {:?}; user: {}({});",
                            self, player_data.name, player_data.id
                        );
                    }
                };
            }
            Some(payload) => {
                let payload = PayloadReader::new(payload);
                match self {
                    id::OSU_SEND_PUBLIC_MESSAGE => {}
                    id::OSU_SEND_PRIVATE_MESSAGE => {}
                    _ => {
                        warn!(
                            "Unhandled packet: {:?}; user: {}({});",
                            self, player_data.name, player_data.id
                        );
                    }
                };
            }
        }
    }
}
