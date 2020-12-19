#![allow(unused_variables)]

use actix_web::web::Data;
use async_std::sync::RwLock;
use log::warn;

use num_traits::FromPrimitive;

use crate::{
    constants::{id, Action, GameMode, PlayMods},
    database::Database,
    events,
    objects::{PlayerData, PlayerSessions},
    packets::{self, HandlerData, PayloadReader},
    types::ChannelList,
};

impl id {
    /// osu!Bancho packet read handle
    pub async fn read_handle(
        &self,
        request_ip: &String,
        token: &String,
        p_data: &PlayerData,
        p_sessions: &Data<RwLock<PlayerSessions>>,
        database: &Data<Database>,
        channel_list: &Data<RwLock<ChannelList>>,
        payload: Option<&[u8]>,
    ) {
        // Data shorthand
        let player_id = p_data.id;
        let player_name = &p_data.name;

        match payload {
            // Payload not exists handlers
            None => {
                match self {
                    id::OSU_PING => {}
                    id::OSU_REQUEST_STATUS_UPDATE => {
                        let p_sessions = p_sessions.read().await;
                        let map = p_sessions.map.read().await;
                        if let Some(player) = map.get(token) {
                            player.enqueue(packets::user_stats(&player).await).await;
                        }
                    }
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
                        events::messages::public(&payload, &channel_list, &p_sessions, &p_data).await
                    }
                    id::OSU_SEND_PRIVATE_MESSAGE => {
                        events::messages::private(&payload, &token, &p_sessions, &p_data).await
                    }
                    id::OSU_USER_STATS_REQUEST => {
                        events::users::stats_request(&payload, &p_sessions, &p_data).await
                    }
                    id::OSU_USER_CHANGE_ACTION => {
                        events::users::change_action(&payload, &token, &p_sessions).await
                    }
                    id::OSU_USER_RECEIVE_UPDATES => {
                        events::users::receive_updates(&payload, &token, &p_sessions).await
                    }
                    id::OSU_USER_FRIEND_ADD => {
                        events::users::add_friend(&payload, &database, &p_data, &token, &p_sessions).await
                    }
                    id::OSU_USER_FRIEND_REMOVE => {
                        events::users::remove_friend(&payload, &database, &p_data, &token, &p_sessions).await
                    }
                    id::OSU_USER_LOGOUT => {
                        // Has payload(i32) but unused
                        events::users::user_logout(&token, &p_sessions, &channel_list).await
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
