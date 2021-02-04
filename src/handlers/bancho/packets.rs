#![allow(unused_variables)]

use actix_web::web::Data;
use async_std::sync::RwLock;
use log::warn;

use num_traits::FromPrimitive;

use crate::{
    constants::{id, Action, GameMode, PlayMod},
    database::Database,
    events,
    objects::{PlayerData, PlayerSessions},
    packets::{self, HandlerContext, PayloadReader},
    types::ChannelList,
};

impl id {
    /// osu!Bancho packet read handle
    pub async fn read_handle<'a>(
        &self,
        request_ip: &'a String,
        token: &'a String,
        data: &'a PlayerData,
        player_sessions: &'a Data<RwLock<PlayerSessions>>,
        database: &'a Data<Database>,
        channel_list: &'a Data<RwLock<ChannelList>>,
        payload: Option<&'a [u8]>,
    ) {
        // Data shorthand
        let player_id = data.id;
        let player_name = &data.name;

        match payload {
            // Payload not exists handlers
            None => {
                match self {
                    id::OSU_PING => {}
                    id::OSU_REQUEST_STATUS_UPDATE => {
                        let player_sessions = player_sessions.read().await;
                        let map = player_sessions.map.read().await;
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
                let ctx = HandlerContext {
                    request_ip,
                    token,
                    id: player_id,
                    name: player_name,
                    data,
                    player_sessions,
                    database,
                    channel_list,
                    payload,
                };
                match self {
                    id::OSU_SEND_PUBLIC_MESSAGE => events::messages::public(&ctx).await,
                    id::OSU_SEND_PRIVATE_MESSAGE => events::messages::private(&ctx).await,
                    id::OSU_USER_STATS_REQUEST => events::users::stats_request(&ctx).await,
                    id::OSU_USER_CHANGE_ACTION => events::users::change_action(&ctx).await,
                    id::OSU_USER_RECEIVE_UPDATES => events::users::receive_updates(&ctx).await,
                    id::OSU_USER_FRIEND_ADD => events::users::add_friend(&ctx).await,
                    id::OSU_USER_FRIEND_REMOVE => events::users::remove_friend(&ctx).await,
                    id::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS => {
                        events::users::toggle_block_non_friend_dms(&ctx).await
                    }
                    id::OSU_USER_CHANNEL_PART => events::users::channel_part(&ctx).await,
                    id::OSU_USER_CHANNEL_JOIN => events::users::channel_join(&ctx).await,
                    id::OSU_USER_LOGOUT => {
                        // Has payload(i32) but unused
                        events::users::user_logout(&ctx).await
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
