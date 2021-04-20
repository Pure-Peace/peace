use std::sync::Weak;

use actix_web::web::Data;
use async_std::sync::RwLock;

use crate::{
    constants::id,
    database::Database,
    events,
    objects::{Bancho, Player, PlayerData},
    packets::HandlerContext,
};

impl id {
    /// osu!Bancho packet read handle
    pub async fn read_handle<'a>(
        &self,
        request_ip: &'a String,
        token: &'a String,
        data: &'a PlayerData,
        weak_player: &'a Weak<RwLock<Player>>,
        bancho: &'a Data<Bancho>,
        database: &'a Data<Database>,
        payload: Option<&'a [u8]>,
    ) {
        // Data shorthand
        let player_id = data.id;
        let player_name = &data.name;
        let player_u_name = &data.u_name;

        match payload {
            // Payload not exists handlers
            None => {
                let build_ctx = || HandlerContext {
                    request_ip,
                    token,
                    id: player_id,
                    name: player_name,
                    u_name: player_u_name,
                    data,
                    weak_player,
                    bancho,
                    database,
                    payload: &[],
                };
                match self {
                    id::OSU_PING => {}
                    id::OSU_USER_REQUEST_STATUS_UPDATE => {
                        events::users::request_status_update(&build_ctx()).await
                    }
                    id::OSU_USER_PRESENCE_REQUEST_ALL => {
                        events::users::presence_request_all(&build_ctx()).await
                    }
                    id::OSU_SPECTATE_STOP => events::spectates::spectate_stop(&build_ctx()).await,
                    id::OSU_SPECTATE_CANT => events::spectates::spectate_cant(&build_ctx()).await,
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
                    u_name: player_u_name,
                    data,
                    weak_player,
                    bancho,
                    database,
                    payload,
                };
                match self {
                    // Messages ---------
                    id::OSU_SEND_PUBLIC_MESSAGE => events::messages::public(&ctx).await,
                    id::OSU_SEND_PRIVATE_MESSAGE => events::messages::private(&ctx).await,
                    // Users ---------
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
                    id::OSU_USER_LOGOUT => events::users::user_logout(&ctx).await,
                    id::OSU_USER_SET_AWAY_MESSAGE => events::users::set_away_message(&ctx).await,
                    id::OSU_USER_PRESENCE_REQUEST => events::users::presence_request(&ctx).await,
                    // Spectates ---------
                    id::OSU_SPECTATE_START => events::spectates::spectate_start(&ctx).await,
                    id::OSU_SPECTATE_FRAMES => {
                        events::spectates::spectate_frames_received(&ctx).await
                    }
                    // TODO: Matches ---------
                    _ => {
                        warn!(
                            "Unhandled packet: {:?}; user: {}({}); payload (length): {:?}",
                            self,
                            player_name,
                            player_id,
                            payload.len()
                        );
                    }
                };
            }
        }
    }
}
