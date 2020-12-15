#![allow(unused_variables)]

use actix_web::web::Data;
use async_std::sync::RwLock;
use log::warn;

use crate::{
    constants::id,
    database::Database,
    events,
    objects::{PlayerData, PlayerSessions},
    packets::{HandlerData, PayloadReader},
    types::ChannelList,
};

impl id {
    /// osu!Bancho packet read handle
    pub async fn read_handle(
        &self,
        request_ip: &String,
        token: &String,
        player: &PlayerData,
        player_sessions: &Data<RwLock<PlayerSessions>>,
        database: &Data<Database>,
        channel_list: &Data<RwLock<ChannelList>>,
        payload: Option<&[u8]>,
    ) {
        // Data shorthand
        let player_id = player.id;
        let player_name = &player.name;

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
                match self {
                    id::OSU_SEND_PUBLIC_MESSAGE => {
                        events::messages::public(&payload, channel_list, player_sessions, player)
                            .await
                    }
                    id::OSU_SEND_PRIVATE_MESSAGE => {
                        events::messages::private(&payload, &token, player_sessions, player).await
                    }
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
                            self, player_name, player_id, payload
                        );
                    }
                };
            }
        }
    }
}
