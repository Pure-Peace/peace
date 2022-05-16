use crate::prelude::*;

#[inline]
/// #0: OSU_USER_CHANGE_ACTION
pub fn user_change_action(
    action: u8,
    info: &str,
    beatmap_md5: &str,
    play_mods_value: u32,
    game_mode: u8,
    beatmap_id: i32,
) -> Vec<u8> {
    build!(
        PacketId::OSU_USER_CHANGE_ACTION,
        data!(
            action,
            info,
            beatmap_md5,
            play_mods_value,
            game_mode,
            beatmap_id
        )
    )
}

#[inline]
/// #1: OSU_SEND_PUBLIC_MESSAGE
pub fn send_public_message(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
    build!(
        PacketId::OSU_SEND_PUBLIC_MESSAGE,
        PacketBuilder::write_message(sender, sender_id, content, target)
    )
}

#[inline]
/// #2: OSU_USER_LOGOUT
pub fn user_logout(user_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_USER_LOGOUT, user_id)
}

#[inline]
/// #3: OSU_USER_REQUEST_STATUS_UPDATE
pub fn user_request_status_update() -> Vec<u8> {
    build!(PacketId::OSU_USER_REQUEST_STATUS_UPDATE)
}

#[inline]
/// #4: OSU_PING
pub fn ping() -> Vec<u8> {
    build!(PacketId::OSU_PING)
}

#[inline]
/// #16: OSU_SPECTATE_START
pub fn spectate_start(target_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_SPECTATE_START, target_id)
}

#[inline]
/// #17: OSU_SPECTATE_STOP
pub fn spectate_stop() -> Vec<u8> {
    build!(PacketId::OSU_SPECTATE_STOP)
}

#[inline]
/// #18: OSU_SPECTATE_FRAMES
pub fn spceate_frames(data: Vec<u8>) -> Vec<u8> {
    build!(PacketId::OSU_SPECTATE_FRAMES, data)
}

#[inline]
/// #20: OSU_ERROR_REPORT
pub fn error_report(data: Vec<u8>) -> Vec<u8> {
    build!(PacketId::OSU_ERROR_REPORT, data)
}

#[inline]
/// #21: OSU_SPECTATE_CANT
pub fn spectate_cant() -> Vec<u8> {
    build!(PacketId::OSU_SPECTATE_CANT)
}

#[inline]
/// #25: OSU_SEND_PRIVATE_MESSAGE
pub fn send_private_message(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
    build!(
        PacketId::OSU_SEND_PRIVATE_MESSAGE,
        PacketBuilder::write_message(sender, sender_id, content, target)
    )
}

#[inline]
/// #29: OSU_USER_PART_LOBBY
pub fn user_part_lobby() -> Vec<u8> {
    build!(PacketId::OSU_USER_PART_LOBBY)
}

#[inline]
/// #30: OSU_USER_JOIN_LOBBY
pub fn user_join_lobby() -> Vec<u8> {
    build!(PacketId::OSU_USER_JOIN_LOBBY)
}

#[inline]
/// #31: OSU_USER_CREATE_MATCH
pub fn user_create_match(
    id: i16,
    in_progress: i8,
    powerplay: i8,
    mods: i32,
    name: &str,
    passwd: &str,
    map_name: &str,
    map_id: i32,
    map_md5: &str,
    slot_statuses: Vec<i8>,
    slot_teams: Vec<i8>,
) -> Vec<u8> {
    build!(
        PacketId::OSU_USER_CREATE_MATCH,
        data!(
            id,
            in_progress,
            powerplay,
            mods,
            name,
            passwd,
            map_name,
            map_id,
            map_md5,
            slot_statuses,
            slot_teams
        )
    )
}

#[inline]
/// #32: OSU_USER_JOIN_MATCH
pub fn user_join_match(match_id: i32, match_password: &str) -> Vec<u8> {
    build!(
        PacketId::OSU_USER_JOIN_MATCH,
        data!(match_id, match_password)
    )
}

#[inline]
/// #33: OSU_USER_PART_MATCH
pub fn user_part_match() -> Vec<u8> {
    build!(PacketId::OSU_USER_PART_MATCH)
}

#[inline]
/// #38: OSU_MATCH_CHANGE_SLOT
pub fn match_change_slot(slot_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_LOCK, slot_id)
}

#[inline]
/// #39: OSU_USER_MATCH_READY
pub fn user_match_ready() -> Vec<u8> {
    build!(PacketId::OSU_USER_MATCH_READY)
}

#[inline]
/// #40: OSU_MATCH_LOCK
pub fn match_lock(slot_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_LOCK, slot_id)
}

#[inline]
/// #41: OSU_MATCH_CHANGE_SETTINGS
pub fn match_change_settings(
    id: i16,
    in_progress: i8,
    powerplay: i8,
    mods: i32,
    name: &str,
    passwd: &str,
    map_name: &str,
    map_id: i32,
    map_md5: &str,
    slot_statuses: Vec<i8>,
    slot_teams: Vec<i8>,
) -> Vec<u8> {
    build!(
        PacketId::OSU_MATCH_CHANGE_SETTINGS,
        data!(
            id,
            in_progress,
            powerplay,
            mods,
            name,
            passwd,
            map_name,
            map_id,
            map_md5,
            slot_statuses,
            slot_teams
        )
    )
}

#[inline]
/// #44: OSU_MATCH_START
pub fn match_start() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_START)
}

#[inline]
/// #47: OSU_MATCH_SCORE_UPDATE
pub fn match_score_update(play_data: Vec<u8>) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_SCORE_UPDATE, play_data)
}

#[inline]
/// #49: OSU_MATCH_COMPLETE
pub fn match_complete() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_COMPLETE)
}

#[inline]
/// #51: OSU_MATCH_CHANGE_MODS
pub fn match_change_mods(mods: i32) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_CHANGE_MODS, mods)
}

#[inline]
/// #52: OSU_MATCH_LOAD_COMPLETE
pub fn match_load_complete() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_LOAD_COMPLETE)
}

#[inline]
/// #54: OSU_MATCH_NO_BEATMAP
pub fn match_no_beatmap() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_NO_BEATMAP)
}

#[inline]
/// #55: OSU_MATCH_NOT_READY
pub fn match_not_ready() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_NOT_READY)
}

#[inline]
/// #56: OSU_MATCH_FAILED
pub fn match_failed() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_FAILED)
}

#[inline]
/// #59: OSU_MATCH_HAS_BEATMAP
pub fn match_has_beatmap() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_HAS_BEATMAP)
}

#[inline]
/// #60: OSU_MATCH_SKIP_REQUEST
pub fn match_skip_request() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_SKIP_REQUEST)
}

#[inline]
/// #63: OSU_USER_CHANNEL_JOIN
pub fn user_channel_join(channel_name: &str) -> Vec<u8> {
    build!(PacketId::OSU_USER_CHANNEL_JOIN, channel_name)
}

#[inline]
/// #68: OSU_BEATMAP_INFO_REQUEST
pub fn beatmap_info_request(beatmap_ids: Vec<i32>) -> Vec<u8> {
    build!(PacketId::OSU_BEATMAP_INFO_REQUEST, beatmap_ids)
}

#[inline]
/// #70: OSU_MATCH_TRANSFER_HOST
pub fn match_transfer_host(slot_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_TRANSFER_HOST, slot_id)
}

#[inline]
/// #73: OSU_USER_FRIEND_ADD
pub fn user_friend_add(target_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_USER_FRIEND_ADD, target_id)
}

#[inline]
/// #74: OSU_USER_FRIEND_REMOVE
pub fn user_friend_remove(target_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_USER_FRIEND_REMOVE, target_id)
}

#[inline]
/// #77: OSU_MATCH_CHANGE_TEAM
pub fn match_change_team() -> Vec<u8> {
    build!(PacketId::OSU_MATCH_CHANGE_TEAM)
}

#[inline]
/// #78: OSU_USER_CHANNEL_PART
pub fn user_channel_part(channel_name: &str) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_CHANGE_TEAM, channel_name)
}

#[inline]
/// #79: OSU_USER_RECEIVE_UPDATES
pub fn user_receive_updates(filter_val: i32) -> Vec<u8> {
    build!(PacketId::OSU_USER_RECEIVE_UPDATES, filter_val)
}

#[inline]
/// #82: OSU_USER_SET_AWAY_MESSAGE
pub fn user_set_away_message(sender: &str, sender_id: i32, content: &str, target: &str) -> Vec<u8> {
    build!(
        PacketId::OSU_USER_SET_AWAY_MESSAGE,
        PacketBuilder::write_message(sender, sender_id, content, target)
    )
}

#[inline]
/// #84: OSU_IRC_ONLY
pub fn irc_only() -> Vec<u8> {
    build!(PacketId::OSU_IRC_ONLY)
}

#[inline]
/// #85: OSU_USER_STATS_REQUEST
pub fn user_stats_request(user_ids: Vec<i32>) -> Vec<u8> {
    build!(PacketId::OSU_USER_STATS_REQUEST, user_ids)
}

#[inline]
/// #87: OSU_MATCH_INVITE
pub fn match_invite(user_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_INVITE, user_id)
}

#[inline]
/// #90: OSU_MATCH_CHANGE_PASSWORD
pub fn match_change_password(password: &str) -> Vec<u8> {
    build!(PacketId::OSU_MATCH_CHANGE_PASSWORD, password)
}

#[inline]
/// #93: OSU_TOURNAMENT_MATCH_INFO_REQUEST
pub fn tournament_match_info_request(match_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_TOURNAMENT_MATCH_INFO_REQUEST, match_id)
}

#[inline]
/// #97: OSU_USER_PRESENCE_REQUEST
pub fn user_presence_request(user_ids: Vec<i32>) -> Vec<u8> {
    build!(PacketId::OSU_USER_PRESENCE_REQUEST, user_ids)
}

#[inline]
/// #98: OSU_USER_PRESENCE_REQUEST_ALL
pub fn user_presence_request_all(ingame_time: i32) -> Vec<u8> {
    build!(PacketId::OSU_USER_PRESENCE_REQUEST_ALL, ingame_time)
}

#[inline]
/// #99: OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS
pub fn user_toggle_block_non_friend_dms(value: i32) -> Vec<u8> {
    build!(PacketId::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS, value)
}

#[inline]
/// #108: OSU_TOURNAMENT_JOIN_MATCH_CHANNEL
pub fn tournament_join_match_channel(match_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_TOURNAMENT_JOIN_MATCH_CHANNEL, match_id)
}

#[inline]
/// #109: OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL
pub fn tournament_leave_match_channel(match_id: i32) -> Vec<u8> {
    build!(PacketId::OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL, match_id)
}
