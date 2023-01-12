use crate::{
    data, packet, write_channel, write_message, LoginResult, MatchData,
    MatchUpdate, PacketId, ScoreFrame, BanchoPacketWrite
};

#[inline]
/// #5: BANCHO_USER_LOGIN_REPLY
pub fn login_reply(login_result: LoginResult) -> Vec<u8> {
    packet!(PacketId::BANCHO_USER_LOGIN_REPLY, login_result)
}

#[inline]
/// #7: BANCHO_SEND_MESSAGE
pub fn send_message(
    sender: &str,
    sender_id: i32,
    content: &str,
    target: &str,
) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_SEND_MESSAGE,
        write_message(sender, sender_id, content, target)
    )
}

#[inline]
/// #8: BANCHO_PONG
pub fn pong() -> Vec<u8> {
    packet!(PacketId::BANCHO_PONG)
}

#[inline]
/// #9: BANCHO_HANDLE_IRC_CHANGE_USERNAME
pub fn change_username(username_old: &str, username_new: &str) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_HANDLE_IRC_CHANGE_USERNAME,
        &format!("{username_old}>>>>{username_new}")
    )
}

#[inline]
/// #11: BANCHO_USER_STATS
pub fn user_stats(
    user_id: i32,
    action: u8,
    info: &str,
    beatmap_md5: &str,
    mods: u32,
    mode: u8,
    beatmap_id: i32,
    ranked_score: i64,
    accuracy: f32,
    playcount: i32,
    total_score: i64,
    rank: i32,
    pp: i16,
) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_USER_STATS,
        data!(
            @capacity { 60 },
            user_id,
            action,
            info,
            beatmap_md5,
            mods,
            mode,
            beatmap_id,
            ranked_score,
            accuracy / 100f32,
            playcount,
            total_score,
            rank,
            pp
        )
    )
}

#[inline]
/// #12: BANCHO_USER_LOGOUT
pub fn user_logout(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_USER_LOGOUT, user_id, 0_u8)
}

#[inline]
/// #13: BANCHO_SPECTATOR_JOINED
pub fn spectator_joined(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_SPECTATOR_JOINED, user_id)
}

#[inline]
/// #14: BANCHO_SPECTATOR_LEFT
pub fn spectator_left(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_SPECTATOR_LEFT, user_id)
}

#[inline]
/// #15: BANCHO_SPECTATE_FRAMES
pub fn spectator_frames(data: Vec<u8>) -> Vec<u8> {
    packet!(PacketId::BANCHO_SPECTATE_FRAMES, data)
}

#[inline]
/// #19: BANCHO_MATCH_JOIN_FAIL
pub fn version_update() -> Vec<u8> {
    packet!(PacketId::BANCHO_VERSION_UPDATE)
}

#[inline]
/// #22: BANCHO_SPECTATOR_CANT_SPECTATE
pub fn spectator_cant_spectate(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_SPECTATOR_CANT_SPECTATE, user_id)
}

#[inline]
/// #23: BANCHO_GET_ATTENTION
pub fn get_attention() -> Vec<u8> {
    packet!(PacketId::BANCHO_GET_ATTENTION)
}

#[inline]
/// #24: BANCHO_NOTIFICATION
pub fn notification(msg: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_NOTIFICATION, msg)
}

#[inline]
/// #26: BANCHO_UPDATE_MATCH
pub fn update_match(match_data: MatchData) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_UPDATE_MATCH,
        MatchUpdate { data: match_data, send_password: false }
    )
}

#[inline]
/// #27: BANCHO_NEW_MATCH
pub fn new_match(match_data: MatchData) -> Vec<u8> {
    packet!(PacketId::BANCHO_NEW_MATCH, match_data)
}

#[inline]
/// #28: BANCHO_DISBAND_MATCH
pub fn disband_match(match_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_DISBAND_MATCH, match_id)
}

#[inline]
/// #34: BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS
pub fn toggle_block_non_friend_pm() -> Vec<u8> {
    packet!(PacketId::BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS)
}

#[inline]
/// #36: BANCHO_MATCH_JOIN_SUCCESS
pub fn match_join_success(match_data: MatchData) -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_JOIN_SUCCESS, match_data)
}

#[inline]
/// #37: BANCHO_MATCH_JOIN_FAIL
pub fn match_join_fail() -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_JOIN_FAIL)
}

#[inline]
/// #42: BANCHO_FELLOW_SPECTATOR_JOINED
pub fn fellow_spectator_joined(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_FELLOW_SPECTATOR_JOINED, user_id)
}

#[inline]
/// #43: BANCHO_FELLOW_SPECTATOR_LEFT
pub fn fellow_spectator_left(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_FELLOW_SPECTATOR_LEFT, user_id)
}

#[inline]
/// #46: BANCHO_MATCH_START
pub fn match_start(match_data: MatchData) -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_START, match_data)
}

#[inline]
/// #48: BANCHO_MATCH_SCORE_UPDATE
pub fn match_score_update(scoreframe: ScoreFrame) -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_START, scoreframe)
}

#[inline]
/// #50: BANCHO_MATCH_TRANSFER_HOST
pub fn match_transfer_host() -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_TRANSFER_HOST)
}

#[inline]
/// #53: BANCHO_MATCH_ALL_PLAYERS_LOADED
pub fn match_all_player_loaded() -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_ALL_PLAYERS_LOADED)
}

#[inline]
/// #57: BANCHO_MATCH_PLAYER_FAILED
pub fn match_player_failed(slot_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_PLAYER_FAILED, slot_id)
}

#[inline]
/// #58: BANCHO_MATCH_COMPLETE
pub fn match_complete() -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_COMPLETE)
}

#[inline]
/// #61: BANCHO_MATCH_SKIP
pub fn match_skip() -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_SKIP)
}

#[inline]
/// #64: BANCHO_CHANNEL_JOIN_SUCCESS
pub fn channel_join(channel_name: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_CHANNEL_JOIN_SUCCESS, channel_name)
}

#[inline]
/// #65: BANCHO_CHANNEL_INFO
pub fn channel_info(name: &str, title: &str, player_count: i16) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_CHANNEL_INFO,
        write_channel(name, title, player_count)
    )
}

#[inline]
/// #66: BANCHO_CHANNEL_KICK
pub fn channel_kick(channel_name: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_CHANNEL_KICK, channel_name)
}

#[inline]
/// #67: BANCHO_CHANNEL_AUTO_JOIN
pub fn channel_auto_join(
    name: &str,
    title: &str,
    player_count: i16,
) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_CHANNEL_AUTO_JOIN,
        write_channel(name, title, player_count)
    )
}

#[inline]
/// #69: BANCHO_BEATMAP_INFO_REPLY
/// UNUSED
pub fn beatmap_info_reply() {
    unimplemented!()
}

#[inline]
/// #71: BANCHO_PRIVILEGES
pub fn bancho_privileges(privileges: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_PRIVILEGES, privileges)
}

#[inline]
/// #72: BANCHO_FRIENDS_LIST
pub fn friends_list(friends: &[i32]) -> Vec<u8> {
    packet!(PacketId::BANCHO_FRIENDS_LIST, friends)
}

#[inline]
/// #75: BANCHO_PROTOCOL_VERSION
pub fn protocol_version(version: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_PROTOCOL_VERSION, version)
}

#[inline]
/// #76: BANCHO_MAIN_MENU_ICON
pub fn main_menu_icon(image_url: &str, link_url: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_MAIN_MENU_ICON, format!("{image_url}|{link_url}"))
}

#[inline]
/// #80: BANCHO_MONITOR
/// deprecated
pub fn monitor() -> Vec<u8> {
    packet!(PacketId::BANCHO_MONITOR)
}

#[inline]
/// #81: BANCHO_MATCH_PLAYER_SKIPPED
pub fn match_player_skipped(slot_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_PLAYER_SKIPPED, slot_id)
}

#[inline]
/// #83: BANCHO_USER_PRESENCE
///
/// including player stats and presence
pub fn user_presence(
    user_id: i32,
    username: &str,
    utc_offset: u8,
    country_code: u8,
    bancho_priv: i32,
    longitude: f32,
    latitude: f32,
    rank: i32,
) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_USER_PRESENCE,
        data!(
            user_id,
            username,
            utc_offset + 24,
            country_code,
            (bancho_priv | 0) as u8,
            longitude,
            latitude,
            rank
        )
    )
}

#[inline]
/// #86: BANCHO_RESTART
pub fn bancho_restart(millis: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_RESTART, millis)
}

#[inline]
/// #88: BANCHO_MATCH_INVITE
pub fn match_invite(
    welcome: &str,
    match_id: i32,
    match_password: Option<&str>,
) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_MATCH_INVITE,
        format!(
            "{welcome} osump://{}/{}.",
            match_id,
            match_password.unwrap_or("")
        )
    )
}

#[inline]
/// #89: BANCHO_CHANNEL_INFO_END
pub fn channel_info_end() -> Vec<u8> {
    packet!(PacketId::BANCHO_CHANNEL_INFO_END)
}

#[inline]
/// #91: BANCHO_MATCH_CHANGE_PASSWORD
pub fn match_change_password(password: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_CHANGE_PASSWORD, password)
}

#[inline]
/// #92: BANCHO_SILENCE_END
pub fn silence_end(duration: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_SILENCE_END, duration)
}

#[inline]
/// #94: BANCHO_USER_SILENCED
pub fn user_silenced(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_USER_SILENCED, user_id)
}

#[inline]
/// #95: BANCHO_USER_PRESENCE_SINGLE
/// UNUSED
pub fn user_presence_single(user_id: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_USER_PRESENCE_SINGLE, user_id)
}

#[inline]
/// #96: BANCHO_USER_PRESENCE_BUNDLE
/// UNUSED
pub fn user_presence_bundle(player_ids: &[i32]) -> Vec<u8> {
    packet!(PacketId::BANCHO_USER_PRESENCE_BUNDLE, player_ids)
}

#[inline]
/// #100: BANCHO_USER_DM_BLOCKED
pub fn user_dm_blocked(target: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_USER_DM_BLOCKED, write_message("", 0, "", target))
}

#[inline]
/// #101: BANCHO_TARGET_IS_SILENCED
pub fn target_silenced(target: &str) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_TARGET_IS_SILENCED,
        write_message("", 0, "", target)
    )
}

#[inline]
/// #102: BANCHO_VERSION_UPDATE_FORCED
pub fn version_update_forced() -> Vec<u8> {
    packet!(PacketId::BANCHO_VERSION_UPDATE_FORCED)
}

#[inline]
/// #103: BANCHO_SWITCH_SERVER
pub fn switch_server(time: i32) -> Vec<u8> {
    packet!(PacketId::BANCHO_SWITCH_SERVER, time)
}

#[inline]
/// #104: BANCHO_ACCOUNT_RESTRICTED
pub fn account_restricted() -> Vec<u8> {
    packet!(PacketId::BANCHO_ACCOUNT_RESTRICTED)
}

#[inline]
/// #105: BANCHO_RTX
/// deprecated
pub fn rtx(msg: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_RTX, msg)
}

#[inline]
/// #106: BANCHO_MATCH_ABORT
pub fn match_abort() -> Vec<u8> {
    packet!(PacketId::BANCHO_MATCH_ABORT)
}

#[inline]
/// #107: BANCHO_SWITCH_TOURNAMENT_SERVER
pub fn switch_tournament_server(ip: &str) -> Vec<u8> {
    packet!(PacketId::BANCHO_SWITCH_TOURNAMENT_SERVER, ip)
}
