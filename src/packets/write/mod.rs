#![allow(dead_code)]
mod tests;

pub mod utils;
pub use utils::*;

use crate::constants::packets::*;

pub type PacketData = Vec<u8>;

/// #5: BANCHO_USER_LOGIN_REPLY
pub fn login_reply(reply: impl LoginReply) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_LOGIN_REPLY)
        .add(write_integer(reply.val()))
        .pack()
}

/// #7: BANCHO_SEND_MESSAGE
pub fn send_message(sender: &str, sender_id: i32, content: &str, channel: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_SEND_MESSAGE)
        .add(write_message(sender, sender_id, content, channel))
        .pack()
}

/// #8: BANCHO_PONG
pub fn pong() -> PacketData {
    simple_pack(id::BANCHO_PONG)
}

/// #9: BANCHO_SEND_MESSAGE
pub fn change_username(username_old: &str, username_new: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_HANDLE_IRC_CHANGE_USERNAME)
        .add(write_string(&format!(
            "{}>>>>{}",
            username_old, username_new
        )))
        .pack()
}

/// #11: BANCHO_USER_STATS
/// TODO
pub fn user_stats() {}

/// #12: BANCHO_USER_LOGOUT
pub fn user_logout(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_LOGOUT)
        .add(write_integer(user_id))
        .add(write_integer::<u8>(0))
        .pack()
}

/// #13: BANCHO_SPECTATOR_JOINED
pub fn specator_joined(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATOR_JOINED)
        .add(write_integer(user_id))
        .pack()
}

/// #14: BANCHO_SPECTATOR_LEFT
pub fn specator_left(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATOR_LEFT)
        .add(write_integer(user_id))
        .pack()
}

/// #15: BANCHO_SPECTATE_FRAMES
/// TODO
pub fn spectator_frames() {}

/// #19: BANCHO_MATCH_JOIN_FAIL
pub fn version_update() -> PacketData {
    simple_pack(id::BANCHO_VERSION_UPDATE)
}

/// #22: BANCHO_SPECTATOR_CANT_SPECTATE
pub fn spectator_cant_spectate(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SPECTATOR_CANT_SPECTATE)
        .add(write_integer(user_id))
        .pack()
}

/// #23: BANCHO_GET_ATTENTION
pub fn get_attention() -> PacketData {
    simple_pack(id::BANCHO_GET_ATTENTION)
}

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
        .add(write_integer(match_id))
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
        .add(write_integer(user_id))
        .pack()
}

/// #43: BANCHO_FELLOW_SPECTATOR_LEFT
pub fn fellow_spectator_left(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_FELLOW_SPECTATOR_LEFT)
        .add(write_integer(user_id))
        .pack()
}

/// #46: BANCHO_MATCH_START
/// TODO
pub fn match_start() {}

/// #48: BANCHO_MATCH_START
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
        .add(write_integer(slot_id))
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
pub fn channel_join(channel: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_NOTIFICATION)
        .add(write_string(channel))
        .pack()
}

/// #65: BANCHO_CHANNEL_INFO
/// TODO
pub fn channel_info() {}

/// #66: BANCHO_CHANNEL_KICK
pub fn channel_kick(channel: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_CHANNEL_KICK)
        .add(write_string(channel))
        .pack()
}

/// #67: BANCHO_CHANNEL_AUTO_JOIN
/// TODO
pub fn channel_auto_join() {}

/// #69: BANCHO_BEATMAP_INFO_REPLY
/// UNUSED
pub fn beatmap_info_reply() {}

/// #71: BANCHO_PRIVILEGES
pub fn bancho_privileges(privileges: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_PRIVILEGES)
        .add(write_integer(privileges))
        .pack()
}

/// #72: BANCHO_FRIENDS_LIST
/// TODO
pub fn friends_list() {}

/// #75: BANCHO_PROTOCOL_VERSION
pub fn protocol_version(version: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_PROTOCOL_VERSION)
        .add(write_integer(version))
        .pack()
}

/// #76: BANCHO_MAIN_MENU_ICON
pub fn main_menu_icon(image_url: &str, click_url: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_MAIN_MENU_ICON)
        .add(write_string(&format!("{}|{}", image_url, click_url)))
        .pack()
}

/// #80: BANCHO_MONITOR
/// deprecated
pub fn monitor() {}

/// #81: BANCHO_MATCH_PLAYER_SKIPPED
pub fn match_player_skipped(slot_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_MATCH_PLAYER_SKIPPED)
        .add(write_integer(slot_id))
        .pack()
}

/// #83: BANCHO_USER_PRESENCE
/// TODO
pub fn user_presence() {}

/// #86: BANCHO_RESTART
pub fn bancho_restart(millis: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_RESTART)
        .add(write_integer(millis))
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
        .add(write_integer(duration))
        .pack()
}

/// #94: BANCHO_USER_SILENCED
pub fn user_silenced(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_SILENCED)
        .add(write_integer(user_id))
        .pack()
}

/// #95: BANCHO_USER_PRESENCE_SINGLE
/// UNUSED
pub fn user_presence_single(user_id: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_PRESENCE_SINGLE)
        .add(write_integer(user_id))
        .pack()
}

/// #96: BANCHO_USER_PRESENCE_BUNDLE
/// UNUSED
pub fn user_presence_bundle() {}

/// #100: BANCHO_USER_DM_BLOCKED
pub fn user_dm_blocked(target: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_USER_DM_BLOCKED)
        .add(write_string(target))
        .pack()
}

/// #101: BANCHO_TARGET_IS_SILENCED
pub fn target_silenced(target: &str) -> PacketData {
    PacketBuilder::with(id::BANCHO_TARGET_IS_SILENCED)
        .add(write_string(target))
        .pack()
}

/// #102: BANCHO_VERSION_UPDATE_FORCED
pub fn version_update_forced() -> PacketData {
    simple_pack(id::BANCHO_VERSION_UPDATE_FORCED)
}

/// #103: BANCHO_SWITCH_SERVER
pub fn switch_server(time: i32) -> PacketData {
    PacketBuilder::with(id::BANCHO_SWITCH_SERVER)
        .add(write_integer(time))
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
