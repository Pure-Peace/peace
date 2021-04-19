use crate::{
    constants::{id, LoginReply},
    objects::{Player, PlayerData},
    types::PacketData,
};

use super::utils::*;

#[inline(always)]
/// #5: BANCHO_USER_LOGIN_REPLY
pub fn login_reply(reply: impl LoginReply) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_LOGIN_REPLY)
        .add(write_num(reply.val()))
        .pack()
}

#[inline(always)]
/// #7: BANCHO_SEND_MESSAGE
pub async fn send_message(sender: &str, sender_id: i32, content: &str, target: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_SEND_MESSAGE)
        .add(write_message(sender, sender_id, content, target).await)
        .pack()
}

#[inline(always)]
/// #8: BANCHO_PONG
pub fn pong() -> PacketData {
    simple_pack(id::BANCHO_PONG)
}

/// #9: BANCHO_HANDLE_IRC_CHANGE_USERNAME
pub fn change_username(username_old: &str, username_new: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_HANDLE_IRC_CHANGE_USERNAME)
        .add(write_string(&format!(
            "{}>>>>{}",
            username_old, username_new
        )))
        .pack()
}

/// #11: BANCHO_USER_STATS
pub async fn user_stats(p: &Player) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_STATS)
        .add_multiple(&mut [
            write_num(p.id),
            write_num(p.status.action as u8),
            write_string(&p.status.info),
            write_string(&p.status.playing_beatmap_md5),
            write_num(p.status.play_mods.value),
            write_num(p.status.game_mode.raw_value()),
            write_num(p.status.playing_beatmap_id),
            write_num(p.stats.ranked_score),
            write_num(p.stats.accuracy),
            write_num(p.stats.playcount),
            write_num(p.stats.total_score),
            write_num(p.stats.rank),
            write_num(p.stats.pp_v2 as i16),
        ])
        .await
        .pack()
}

pub async fn user_stats_from_data(p: &PlayerData) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_STATS)
        .add_multiple(&mut [
            write_num(p.id),
            write_num(p.status.action as u8),
            write_string(&p.status.info),
            write_string(&p.status.playing_beatmap_md5),
            write_num(p.status.play_mods.value),
            write_num(p.status.game_mode.raw_value()),
            write_num(p.status.playing_beatmap_id),
            write_num(p.stats.ranked_score),
            write_num(p.stats.accuracy),
            write_num(p.stats.playcount),
            write_num(p.stats.total_score),
            write_num(p.stats.rank),
            write_num(p.stats.pp_v2 as i16),
        ])
        .await
        .pack()
}

#[inline(always)]
/// #12: BANCHO_USER_LOGOUT
pub fn user_logout(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_LOGOUT)
        .add(write_num(user_id))
        .add(write_num::<u8>(0))
        .pack()
}

/// #13: BANCHO_SPECTATOR_JOINED
pub fn spectator_joined(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATOR_JOINED)
        .add(write_num(user_id))
        .pack()
}

/// #14: BANCHO_SPECTATOR_LEFT
pub fn spectator_left(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATOR_LEFT)
        .add(write_num(user_id))
        .pack()
}

/// #15: BANCHO_SPECTATE_FRAMES
pub fn spectator_frames(data: PacketData) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATE_FRAMES)
        .add(data)
        .pack()
}

/// #19: BANCHO_MATCH_JOIN_FAIL
pub fn version_update() -> PacketData {
    simple_pack(id::BANCHO_VERSION_UPDATE)
}

/// #22: BANCHO_SPECTATOR_CANT_SPECTATE
pub fn spectator_cant_spectate(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATOR_CANT_SPECTATE)
        .add(write_num(user_id))
        .pack()
}

/// #23: BANCHO_GET_ATTENTION
pub fn get_attention() -> PacketData {
    simple_pack(id::BANCHO_GET_ATTENTION)
}

#[inline(always)]
/// #24: BANCHO_NOTIFICATION
pub fn notification(msg: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_NOTIFICATION)
        .add(write_string(msg))
        .pack()
}

/// #26: BANCHO_UPDATE_MATCH
/// TODO
pub fn update_match() {}

/// #27: BANCHO_NEW_MATCH
/// TODO
pub fn new_match() {}

/// #28: BANCHO_DISBAND_MATCH
pub fn disband_match(match_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_DISBAND_MATCH)
        .add(write_num(match_id))
        .pack()
}

/// #34: BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS
pub fn toggle_block_non_friend_pm() -> PacketData {
    simple_pack(id::BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS)
}

/// #36: BANCHO_MATCH_JOIN_SUCCESS
/// TODO
pub fn match_join_success() {}

/// #37: BANCHO_MATCH_JOIN_FAIL
pub fn match_join_fail() -> PacketData {
    simple_pack(id::BANCHO_MATCH_JOIN_FAIL)
}

/// #42: BANCHO_FELLOW_SPECTATOR_JOINED
pub fn fellow_spectator_joined(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_FELLOW_SPECTATOR_JOINED)
        .add(write_num(user_id))
        .pack()
}

/// #43: BANCHO_FELLOW_SPECTATOR_LEFT
pub fn fellow_spectator_left(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_FELLOW_SPECTATOR_LEFT)
        .add(write_num(user_id))
        .pack()
}

/// #46: BANCHO_MATCH_START
/// TODO
pub fn match_start() {}

/// #48: BANCHO_MATCH_SCORE_UPDATE
/// TODO
pub fn match_score_update() {}

/// #50: BANCHO_MATCH_TRANSFER_HOST
pub fn match_transfer_host() -> PacketData {
    simple_pack(id::BANCHO_MATCH_TRANSFER_HOST)
}

/// #53: BANCHO_MATCH_ALL_PLAYERS_LOADED
pub fn match_all_player_loaded() -> PacketData {
    simple_pack(id::BANCHO_MATCH_ALL_PLAYERS_LOADED)
}

/// #57: BANCHO_MATCH_PLAYER_FAILED
pub fn match_player_failed(slot_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_MATCH_PLAYER_FAILED)
        .add(write_num(slot_id))
        .pack()
}

/// #58: BANCHO_MATCH_COMPLETE
pub fn match_complete() -> PacketData {
    simple_pack(id::BANCHO_MATCH_COMPLETE)
}

/// #61: BANCHO_MATCH_SKIP
pub fn match_skip() -> PacketData {
    simple_pack(id::BANCHO_MATCH_SKIP)
}

/// #64: BANCHO_CHANNEL_JOIN_SUCCESS
pub fn channel_join(channel_name: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_CHANNEL_JOIN_SUCCESS)
        .add(write_string(channel_name))
        .pack()
}

/// #65: BANCHO_CHANNEL_INFO
pub fn channel_info(name: &str, title: &str, player_count: i16) -> PacketData {
    PacketBuilder::with(id::BANCHO_CHANNEL_INFO)
        .add(write_channel(name, title, player_count))
        .pack()
}

/// #66: BANCHO_CHANNEL_KICK
pub fn channel_kick(channel_name: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_CHANNEL_KICK)
        .add(write_string(channel_name))
        .pack()
}

/// #67: BANCHO_CHANNEL_AUTO_JOIN
pub fn channel_auto_join(name: &str, title: &str, player_count: i16) -> PacketData {
    PacketBuilder::with(id::BANCHO_CHANNEL_AUTO_JOIN)
        .add(write_channel(name, title, player_count))
        .pack()
}

/// #69: BANCHO_BEATMAP_INFO_REPLY
/// UNUSED
pub fn beatmap_info_reply() {}

/// #71: BANCHO_PRIVILEGES
pub fn bancho_privileges(privileges: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_PRIVILEGES)
        .add(write_num(privileges))
        .pack()
}

/// #72: BANCHO_FRIENDS_LIST
pub async fn friends_list(friends: &Vec<i32>) -> PacketData {
    PacketBuilder::with(id::BANCHO_FRIENDS_LIST)
        .add(write_int_list(friends).await)
        .pack()
}

/// #75: BANCHO_PROTOCOL_VERSION
pub fn protocol_version(version: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_PROTOCOL_VERSION)
        .add(write_num(version))
        .pack()
}

#[inline(always)]
/// #76: BANCHO_MAIN_MENU_ICON
/// menu_icon = image_url: &str | click_url: &str
pub fn main_menu_icon(menu_icon: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_MAIN_MENU_ICON)
        .add(write_string(menu_icon))
        .pack()
}

/// #80: BANCHO_MONITOR
/// deprecated
pub fn monitor() {}

/// #81: BANCHO_MATCH_PLAYER_SKIPPED
pub fn match_player_skipped(slot_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_MATCH_PLAYER_SKIPPED)
        .add(write_num(slot_id))
        .pack()
}

/// #83: BANCHO_USER_PRESENCE
///
/// including player stats and presence
pub async fn user_presence(p: &Player) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_PRESENCE)
        .add_multiple(&mut [
            write_num(p.id),
            write_string(&p.name),
            write_num(p.utc_offset + 24),
            write_num(p.get_country_code()),
            write_num((p.bancho_privileges | 0) as u8),
            write_num(p.geo_data.longitude as f32),
            write_num(p.geo_data.latitude as f32),
            write_num(p.stats.rank),
        ])
        .await
        .pack()
}

/// #83: BANCHO_USER_PRESENCE
///
/// including player stats and presence
pub async fn user_presence_from_data(p: &PlayerData) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_PRESENCE)
        .add_multiple(&mut [
            write_num(p.id),
            write_string(&p.name),
            write_num(p.utc_offset + 24),
            write_num(p.get_country_code()),
            write_num((p.bancho_privileges | 0) as u8),
            write_num(p.geo_data.longitude as f32),
            write_num(p.geo_data.latitude as f32),
            write_num(p.stats.rank),
        ])
        .await
        .pack()
}

/// #86: BANCHO_RESTART
pub fn bancho_restart(millis: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_RESTART)
        .add(write_num(millis))
        .pack()
}

/// #88: BANCHO_MATCH_INVITE
/// TODO
pub fn match_invite() {}

/// #89: BANCHO_CHANNEL_INFO_END
pub fn channel_info_end() -> PacketData {
    simple_pack(id::BANCHO_CHANNEL_INFO_END)
}

/// #91: BANCHO_MATCH_CHANGE_PASSWORD
pub fn match_change_password(password: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_MATCH_CHANGE_PASSWORD)
        .add(write_string(password))
        .pack()
}

/// #92: BANCHO_SILENCE_END
pub fn silence_end(duration: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SILENCE_END)
        .add(write_num(duration))
        .pack()
}

/// #94: BANCHO_USER_SILENCED
pub fn user_silenced(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_SILENCED)
        .add(write_num(user_id))
        .pack()
}

/// #95: BANCHO_USER_PRESENCE_SINGLE
/// UNUSED
pub fn user_presence_single(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_PRESENCE_SINGLE)
        .add(write_num(user_id))
        .pack()
}

/// #96: BANCHO_USER_PRESENCE_BUNDLE
/// UNUSED
pub async fn user_presence_bundle(player_id_list: &Vec<i32>) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_PRESENCE_BUNDLE)
        .add(write_int_list(player_id_list).await)
        .pack()
}

/// #100: BANCHO_USER_DM_BLOCKED
pub fn user_dm_blocked(target: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_DM_BLOCKED)
        .add(write_message_sync("", 0, "", target))
        .pack()
}

/// #101: BANCHO_TARGET_IS_SILENCED
pub fn target_silenced(target: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_TARGET_IS_SILENCED)
        .add(write_message_sync("", 0, "", target))
        .pack()
}

/// #102: BANCHO_VERSION_UPDATE_FORCED
pub fn version_update_forced() -> PacketData {
    simple_pack(id::BANCHO_VERSION_UPDATE_FORCED)
}

/// #103: BANCHO_SWITCH_SERVER
pub fn switch_server(time: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SWITCH_SERVER)
        .add(write_num(time))
        .pack()
}

/// #104: BANCHO_ACCOUNT_RESTRICTED
pub fn account_restricted() -> PacketData {
    simple_pack(id::BANCHO_ACCOUNT_RESTRICTED)
}

/// #105: BANCHO_RTX
/// deprecated
pub fn rtx(msg: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_RTX)
        .add(write_string(msg))
        .pack()
}

/// #106: BANCHO_MATCH_ABORT
pub fn match_abort() -> PacketData {
    simple_pack(id::BANCHO_MATCH_ABORT)
}

/// #107: BANCHO_SWITCH_TOURNAMENT_SERVER
pub fn switch_tournament_server(ip: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_SWITCH_TOURNAMENT_SERVER)
        .add(write_string(ip))
        .pack()
}

#[inline(always)]
/// #83 + #11: USER_DATA_PACKETDATA
pub async fn user_data(p: &Player) -> PacketData {
    PacketBuilder::from_multiple(&mut [user_presence(&p).await, user_stats(&p).await])
        .await
        .write_out()
}
