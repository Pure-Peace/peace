use crate::*;

packet_struct!(
    PacketId::OSU_USER_CHANGE_ACTION,
    /// #0: OSU_USER_CHANGE_ACTION
    UserChangeAction<'a> {
        online_status: u8,
        description: CowStr<'a>,
        beatmap_md5: CowStr<'a>,
        mods: u32,
        mode: u8,
        beatmap_id: i32,
    }
);

packet_struct!(
    PacketId::OSU_SEND_PUBLIC_MESSAGE,
    /// #1: OSU_SEND_PUBLIC_MESSAGE
    SendPublicMessage<'a> {
        sender: CowStr<'a>,
        content: CowStr<'a>,
        target: CowStr<'a>,
        sender_id: i32,
    }
);

packet_struct!(
    PacketId::OSU_USER_LOGOUT,
    /// #2: OSU_USER_LOGOUT
    UserLogout { user_id: i32 }
);

packet_struct!(
    PacketId::OSU_USER_REQUEST_STATUS_UPDATE,
    /// #3: OSU_USER_REQUEST_STATUS_UPDATE
    UserRequestStatusUpdate {}
);

packet_struct!(
    PacketId::OSU_PING,
    /// #4: OSU_PING
    Ping {}
);

packet_struct!(
    PacketId::OSU_SPECTATE_START,
    /// #16: OSU_SPECTATE_START
    SpectateStart { target_id: i32 }
);

packet_struct!(
    PacketId::OSU_SPECTATE_STOP,
    /// #17: OSU_SPECTATE_STOP
    SpectateStop {}
);

packet_struct!(
    PacketId::OSU_SPECTATE_FRAMES,
    /// #18: OSU_SPECTATE_FRAMES
    SpceateFrames { data: Vec<u8> }
);

packet_struct!(
    PacketId::OSU_ERROR_REPORT,
    /// #20: OSU_ERROR_REPORT
    ErrorReport { data: Vec<u8> }
);

packet_struct!(
    PacketId::OSU_SPECTATE_CANT,
    /// #21: OSU_SPECTATE_CANT
    SpectateCant {}
);

packet_struct!(
    PacketId::OSU_SEND_PRIVATE_MESSAGE,
    /// #25: OSU_SEND_PRIVATE_MESSAGE
    SendPrivateMessage<'a> {
        sender: CowStr<'a>,
        content: CowStr<'a>,
        target: CowStr<'a>,
        sender_id: i32,
    }
);

packet_struct!(
    PacketId::OSU_USER_PART_LOBBY,
    /// #29: OSU_USER_PART_LOBBY
    UserPartLobby {}
);

packet_struct!(
    PacketId::OSU_USER_JOIN_LOBBY,
    /// #30: OSU_USER_JOIN_LOBBY
    UserJoinLobby {}
);

packet_struct!(
    PacketId::OSU_USER_CREATE_MATCH,
    /// #31: OSU_USER_CREATE_MATCH
    UserCreateMatch<'a> {
        id: i16,
        in_progress: i8,
        powerplay: i8,
        mods: i32,
        name: CowStr<'a>,
        passwd: CowStr<'a>,
        map_name: CowStr<'a>,
        map_id: i32,
        map_md5: CowStr<'a>,
        slot_statuses: &'a [i8],
        slot_teams: &'a [i8],
    }
);

packet_struct!(
    PacketId::OSU_USER_JOIN_MATCH,
    /// #32: OSU_USER_JOIN_MATCH
    UserJoinMatch<'a> {
        match_id: i32,
        match_password: CowStr<'a>
    }
);

packet_struct!(
    PacketId::OSU_USER_PART_MATCH,
    /// #33: OSU_USER_PART_MATCH
    UserPartMatch {}
);

packet_struct!(
    PacketId::OSU_MATCH_CHANGE_SLOT,
    /// #38: OSU_MATCH_CHANGE_SLOT
    MatchChangeSlot { slot_id: i32 }
);

packet_struct!(
    PacketId::OSU_USER_MATCH_READY,
    /// #39: OSU_USER_MATCH_READY
    UserMatchReady {}
);

packet_struct!(
    PacketId::OSU_MATCH_LOCK,
    /// #40: OSU_MATCH_LOCK
    MatchLock { slot_id: i32 }
);

packet_struct!(
    PacketId::OSU_MATCH_CHANGE_SETTINGS,
    /// #41: OSU_MATCH_CHANGE_SETTINGS
    MatchChangeSettings<'a> {
        id: i16,
        in_progress: i8,
        powerplay: i8,
        mods: i32,
        name: CowStr<'a>,
        passwd: CowStr<'a>,
        map_name: CowStr<'a>,
        map_id: i32,
        map_md5: CowStr<'a>,
        slot_statuses: &'a [i8],
        slot_teams: &'a [i8],
    }
);

packet_struct!(
    PacketId::OSU_MATCH_START,
    /// #44: OSU_MATCH_START
    MatchStart {}
);

packet_struct!(
    PacketId::OSU_MATCH_SCORE_UPDATE,
    /// #47: OSU_MATCH_SCORE_UPDATE
    MatchScoreUpdate {
        play_data: Vec<u8>
    }
);

packet_struct!(
    PacketId::OSU_MATCH_COMPLETE,
    /// #49: OSU_MATCH_COMPLETE
    MatchComplete {}
);

packet_struct!(
    PacketId::OSU_MATCH_CHANGE_MODS,
    /// #51: OSU_MATCH_CHANGE_MODS
    MatchChangeMods { mods: i32 }
);

packet_struct!(
    PacketId::OSU_MATCH_LOAD_COMPLETE,
    /// #52: OSU_MATCH_LOAD_COMPLETE
    MatchLoadComplete {}
);

packet_struct!(
    PacketId::OSU_MATCH_NO_BEATMAP,
    /// #54: OSU_MATCH_NO_BEATMAP
    MatchNoBeatmap {}
);

packet_struct!(
    PacketId::OSU_MATCH_NOT_READY,
    /// #55: OSU_MATCH_NOT_READY
    MatchNotReady {}
);

packet_struct!(
    PacketId::OSU_MATCH_FAILED,
    /// #56: OSU_MATCH_FAILED
    MatchFailed {}
);

packet_struct!(
    PacketId::OSU_MATCH_HAS_BEATMAP,
    /// #59: OSU_MATCH_HAS_BEATMAP
    MatchHasBeatmap {}
);

packet_struct!(
    PacketId::OSU_MATCH_SKIP_REQUEST,
    /// #60: OSU_MATCH_SKIP_REQUEST
    MatchSkipRequest {}
);

packet_struct!(
    PacketId::OSU_USER_CHANNEL_JOIN,
    /// #63: OSU_USER_CHANNEL_JOIN
    UserChannelJoin<'a> {
        channel_name: CowStr<'a>
    }
);

packet_struct!(
    PacketId::OSU_BEATMAP_INFO_REQUEST,
    /// #68: OSU_BEATMAP_INFO_REQUEST
    BeatmapInfoRequest<'a> {
        beatmap_ids: &'a [i32]
    }
);

packet_struct!(
    PacketId::OSU_MATCH_TRANSFER_HOST,
    /// #70: OSU_MATCH_TRANSFER_HOST
    MatchTransferHost { slot_id: i32 }
);

packet_struct!(
    PacketId::OSU_USER_FRIEND_ADD,
    /// #73: OSU_USER_FRIEND_ADD
    UserFriendAdd { target_id: i32 }
);

packet_struct!(
    PacketId::OSU_USER_FRIEND_REMOVE,
    /// #74: OSU_USER_FRIEND_REMOVE
    UserFriendRemove { target_id: i32 }
);

packet_struct!(
    PacketId::OSU_MATCH_CHANGE_TEAM,
    /// #77: OSU_MATCH_CHANGE_TEAM
    MatchChangeTeam {}
);

packet_struct!(
    PacketId::OSU_USER_CHANNEL_PART,
    /// #78: OSU_USER_CHANNEL_PART
    UserChannelPart<'a> {
        channel_name: CowStr<'a>
    }
);

packet_struct!(
    PacketId::OSU_USER_RECEIVE_UPDATES,
    /// #79: OSU_USER_RECEIVE_UPDATES
    UserReceiveUpdates { filter_val: i32 }
);

packet_struct!(
    PacketId::OSU_USER_SET_AWAY_MESSAGE,
    /// #82: OSU_USER_SET_AWAY_MESSAGE
    UserSetAwayMessage<'a> {
        sender: CowStr<'a>,
        content: CowStr<'a>,
        target: CowStr<'a>,
        sender_id: i32,
    }
);

packet_struct!(
    PacketId::OSU_IRC_ONLY,
    /// #84: OSU_IRC_ONLY
    IrcOnly {}
);

packet_struct!(
    PacketId::OSU_USER_STATS_REQUEST,
    /// #85: OSU_USER_STATS_REQUEST
    UserStatsRequest<'a> {
        user_ids: &'a [i32]
    }
);

packet_struct!(
    PacketId::OSU_MATCH_INVITE,
    /// #87: OSU_MATCH_INVITE
    MatchInvite { user_id: i32 }
);

packet_struct!(
    PacketId::OSU_MATCH_CHANGE_PASSWORD,
    /// #90: OSU_MATCH_CHANGE_PASSWORD
    MatchChangePassword<'a> {
        password: CowStr<'a>
    }
);

packet_struct!(
    PacketId::OSU_TOURNAMENT_MATCH_INFO_REQUEST,
    /// #93: OSU_TOURNAMENT_MATCH_INFO_REQUEST
    TournamentMatchInfoRequest { match_id: i32 }
);

packet_struct!(
    PacketId::OSU_USER_PRESENCE_REQUEST,
    /// #97: OSU_USER_PRESENCE_REQUEST
    UserPresenceRequest<'a> {
        user_ids: &'a [i32]
    }
);

packet_struct!(
    PacketId::OSU_USER_PRESENCE_REQUEST_ALL,
    /// #98: OSU_USER_PRESENCE_REQUEST_ALL
    UserPresenceRequestAll { ingame_time: i32 }
);

packet_struct!(
    PacketId::OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS,
    /// #99: OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS
    UserToggleBlockNonFriendDms { value: i32 }
);

packet_struct!(
    PacketId::OSU_TOURNAMENT_JOIN_MATCH_CHANNEL,
    /// #108: OSU_TOURNAMENT_JOIN_MATCH_CHANNEL
    TournamentJoinMatchChannel { match_id: i32 }
);

packet_struct!(
    PacketId::OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL,
    /// #109: OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL
    TournamentLeaveMatchChannel { match_id: i32 }
);
