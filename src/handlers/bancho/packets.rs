#![allow(unused_variables)]

use actix_web::web::Data;
use async_std::sync::RwLock;
use log::warn;

use crate::{
    constants::id,
    database::Database,
    objects::{PlayerData, PlayerSessions},
    packets::{HandlerData, PayloadReader},
    types::ChannelList,
};

impl id {
    /// osu!Bancho packet read handle
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
        // Data shorthand
        let player_id = player_data.id;
        let player_name = &player_data.name;

        match payload {
            // Payload not exists handlers
            None => {
                match self {
                    id::OSU_PING => {}
                    _ => {
                        warn!(
                            "Unhandled packet (Non-payload): {:?}; user: {}({});",
                            self, player_name, player_id
                        );
                    }
                };
            }
            // Payload exists handlers
            Some(payload) => {
                let mut payload = PayloadReader::new(payload);
                match self {
                    id::OSU_SEND_PUBLIC_MESSAGE => {
                        let message = payload.read_message().await;

                        match channel_list.read().await.get(&message.target) {
                            Some(channel) => {
                                channel
                                    .broadcast(player_name, player_id, &message.content, false)
                                    .await;
                                info!(
                                    "{}({}) <pub>@ {}: {}",
                                    player_name, player_id, channel.name, message.content
                                );
                            }
                            None => {
                                warn!(
                                    "Player {}({}) try send message to non-existent channel: {}",
                                    player_name, player_id, message.target
                                );
                                return;
                            }
                        }
                    }
                    id::OSU_SEND_PRIVATE_MESSAGE => {}
                    id::OSU_LOGOUT => {
                        // Has payload: integer (len = 4)
                        player_sessions
                            .write()
                            .await
                            .logout(token, Some(channel_list))
                            .await;
                    }
                    _ => {
                        warn!(
                            "Unhandled packet: {:?}; user: {}({}); payload: {:?}",
                            self, player_name, player_id, payload.payload
                        );
                    }
                };
            }
        }
    }
}
