use std::sync::Weak;

use ntex::web::types::Data;
use async_std::sync::RwLock;

use crate::{
    events,
    objects::{Bancho, Player, PlayerData},
};

use peace_constants::packets::id;
use peace_database::Database;

use super::HandlerContext;

/// osu!Bancho packet read handle
#[inline(always)]
pub async fn read_handle<'a>(
    packet_id: &'a id,
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
                payload: &[],
            };
            match packet_id {
                // Users ---------
                id::OSU_USER_REQUEST_STATUS_UPDATE => {
                    events::users::request_status_update(&ctx).await
                }
                id::OSU_USER_PRESENCE_REQUEST_ALL => {
                    events::users::presence_request_all(&ctx).await
                }
                id::OSU_SPECTATE_STOP => events::spectates::spectate_stop(&ctx).await,
                id::OSU_SPECTATE_CANT => events::spectates::spectate_cant(&ctx).await,
                // TODO: User.matches ---------
                id::OSU_USER_PART_LOBBY => events::users::lobby_part(&ctx).await,
                id::OSU_USER_JOIN_LOBBY => events::users::lobby_join(&ctx).await,
                /* id::OSU_USER_PART_MATCH => events::users::match_part(&ctx).await,
                id::OSU_USER_MATCH_READY => events::users::match_ready(&ctx).await,
                // Matches ---------
                id::OSU_MATCH_START => events::matches::start(&ctx).await,
                id::OSU_MATCH_COMPLETE => events::matches::complete(&ctx).await,
                id::OSU_MATCH_LOAD_COMPLETE => events::matches::load_complete(&ctx).await,
                id::OSU_MATCH_NO_BEATMAP => events::matches::no_beatmap(&ctx).await,
                id::OSU_MATCH_NOT_READY => events::matches::not_ready(&ctx).await,
                id::OSU_MATCH_FAILED => events::matches::failed(&ctx).await,
                id::OSU_MATCH_HAS_BEATMAP => events::matches::has_beatmap(&ctx).await,
                id::OSU_MATCH_SKIP_REQUEST => events::matches::skip_request(&ctx).await,
                id::OSU_MATCH_CHANGE_TEAM => events::matches::change_team(&ctx).await, */
                _ => {
                    warn!(
                        "Unhandled packet (Non-payload): {:?}; user: {}({});",
                        packet_id, player_name, player_id
                    );
                    None
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
            match packet_id {
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
                id::OSU_SPECTATE_FRAMES => events::spectates::spectate_frames_received(&ctx).await,
                /* // TODO: User.matches ---------
                id::OSU_USER_CREATE_MATCH => events::users::match_create(&ctx).await,
                id::OSU_USER_JOIN_MATCH => events::users::match_join(&ctx).await,
                // TODO: Matches ---------
                id::OSU_MATCH_CHANGE_SLOT => events::matches::change_slot(&ctx).await,
                id::OSU_MATCH_LOCK => events::matches::lock(&ctx).await,
                id::OSU_MATCH_CHANGE_SETTINGS => events::matches::change_settings(&ctx).await,
                id::OSU_MATCH_SCORE_UPDATE => events::matches::score_update(&ctx).await,
                id::OSU_MATCH_CHANGE_MODS => events::matches::change_mods(&ctx).await,
                id::OSU_MATCH_TRANSFER_HOST => events::matches::transfer_host(&ctx).await,
                id::OSU_MATCH_INVITE => events::matches::invite(&ctx).await,
                id::OSU_MATCH_CHANGE_PASSWORD => events::matches::change_password(&ctx).await,
                // TODO: Tournament
                id::OSU_TOURNAMENT_MATCH_INFO_REQUEST => {
                    events::tournaments::match_info_request(&ctx).await
                }
                id::OSU_TOURNAMENT_JOIN_MATCH_CHANNEL => {
                    events::tournaments::join_match_channel(&ctx).await
                }
                id::OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL => {
                    events::tournaments::leave_match_channel(&ctx).await
                } */
                _ => {
                    warn!(
                        "Unhandled packet: {:?}; user: {}({}); payload (length): {:?}",
                        packet_id,
                        player_name,
                        player_id,
                        payload.len()
                    );
                    None
                }
            };
        }
    }
}
