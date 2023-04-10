use crate::*;

packet_struct!(
    PacketId::BANCHO_USER_LOGIN_REPLY,
    /// #5: BANCHO_USER_LOGIN_REPLY
    LoginReply { login_result: LoginResult }
);

packet_struct!(
    PacketId::BANCHO_SEND_MESSAGE,
    /// #7: BANCHO_SEND_MESSAGE
    SendMessage<'a> {
        sender: CowStr<'a>,
        content: CowStr<'a>,
        target: CowStr<'a>,
        sender_id: i32,
    }
);

packet_struct!(
    PacketId::BANCHO_PONG,
    /// #8: BANCHO_PONG
    Pong {}
);

packet_struct!(
    PacketId::BANCHO_HANDLE_IRC_CHANGE_USERNAME,
    /// #9: BANCHO_HANDLE_IRC_CHANGE_USERNAME
    ChangeUsername<'a> {
        username_old: CowStr<'a>,
        username_new: CowStr<'a>,
    },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(
            Self::ID,
            format!("{}>>>>{}", self.username_old, self.username_new)
        )
    }
);

packet_struct!(
    PacketId::BANCHO_USER_STATS,
    /// #11: BANCHO_USER_STATS
    UserStats<'a> {
        user_id: i32,
        online_status: u8,
        description: CowStr<'a>,
        beatmap_md5: CowStr<'a>,
        mods: u32,
        mode: u8,
        beatmap_id: i32,
        ranked_score: i64,
        accuracy: f32,
        playcount: i32,
        total_score: i64,
        rank: i32,
        pp: i16,
    },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(
            Self::ID,
            self.user_id,
            self.online_status,
            self.description,
            self.beatmap_md5,
            self.mods,
            self.mode,
            self.beatmap_id,
            self.ranked_score,
            self.accuracy / 100f32,
            self.playcount,
            self.total_score,
            self.rank,
            self.pp
        )
    }
);

packet_struct!(
    PacketId::BANCHO_USER_LOGOUT,
    /// #12: BANCHO_USER_LOGOUT
    UserLogout { user_id: i32 },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(Self::ID, self.user_id, 0_u8)
    }
);

packet_struct!(
    PacketId::BANCHO_SPECTATOR_JOINED,
    /// #13: BANCHO_SPECTATOR_JOINED
    SpectatorJoined { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_SPECTATOR_LEFT,
    /// #14: BANCHO_SPECTATOR_LEFT
    SpectatorLeft { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_SPECTATE_FRAMES,
    /// #15: BANCHO_SPECTATE_FRAMES
    SpectatorFrames { data: Vec<u8> }
);

packet_struct!(
    PacketId::BANCHO_VERSION_UPDATE,
    /// #19: BANCHO_VERSION_UPDATE
    VersionUpdate {}
);

packet_struct!(
    PacketId::BANCHO_SPECTATOR_CANT_SPECTATE,
    /// #22: BANCHO_SPECTATOR_CANT_SPECTATE
    SpectatorCantSpectate { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_GET_ATTENTION,
    /// #23: BANCHO_GET_ATTENTION
    GetAttention {}
);

packet_struct!(
    PacketId::BANCHO_NOTIFICATION,
    /// #24: BANCHO_NOTIFICATION
    Notification<'a> { msg: CowStr<'a> }
);

packet_struct!(
    PacketId::BANCHO_UPDATE_MATCH,
    /// #26: BANCHO_UPDATE_MATCH
    UpdateMatch { match_data: MatchData },
    fn into_packet_data(self) -> Vec<u8> {
        let data = MatchUpdate { data: self.match_data, send_password: false };
        packet!(Self::ID, data)
    }
);

packet_struct!(
    PacketId::BANCHO_NEW_MATCH,
    /// #27: BANCHO_NEW_MATCH
    NewMatch { match_data: MatchData }
);

packet_struct!(
    PacketId::BANCHO_DISBAND_MATCH,
    /// #28: BANCHO_DISBAND_MATCH
    DisbandMatch { match_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS,
    /// #34: BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS
    ToggleBlockNonFriendPm {}
);

packet_struct!(
    PacketId::BANCHO_MATCH_JOIN_SUCCESS,
    /// #36: BANCHO_MATCH_JOIN_SUCCESS
    MatchJoinSuccess { match_data: MatchData }
);

packet_struct!(
    PacketId::BANCHO_MATCH_JOIN_FAIL,
    /// #37: BANCHO_MATCH_JOIN_FAIL
    MatchJoinFail {}
);

packet_struct!(
    PacketId::BANCHO_FELLOW_SPECTATOR_JOINED,
    /// #42: BANCHO_FELLOW_SPECTATOR_JOINED
    FellowSpectatorJoined { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_FELLOW_SPECTATOR_LEFT,
    /// #43: BANCHO_FELLOW_SPECTATOR_LEFT
    FellowSpectatorLeft { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_MATCH_START,
    /// #46: BANCHO_MATCH_START
    MatchStart { match_data: MatchData }
);

packet_struct!(
    PacketId::BANCHO_MATCH_SCORE_UPDATE,
    /// #48: BANCHO_MATCH_SCORE_UPDATE
    MatchScoreUpdate { scoreframe: ScoreFrame }
);

packet_struct!(
    PacketId::BANCHO_MATCH_TRANSFER_HOST,
    /// #50: BANCHO_MATCH_TRANSFER_HOST
    MatchTransferHost {}
);

packet_struct!(
    PacketId::BANCHO_MATCH_ALL_PLAYERS_LOADED,
    /// #53: BANCHO_MATCH_ALL_PLAYERS_LOADED
    MatchAllPlayerLoaded {}
);

packet_struct!(
    PacketId::BANCHO_MATCH_PLAYER_FAILED,
    /// #57: BANCHO_MATCH_PLAYER_FAILED
    MatchPlayerFailed { slot_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_MATCH_COMPLETE,
    /// #58: BANCHO_MATCH_COMPLETE
    MatchComplete {}
);

packet_struct!(
    PacketId::BANCHO_MATCH_SKIP,
    /// #61: BANCHO_MATCH_SKIP
    MatchSkip {}
);

packet_struct!(
    PacketId::BANCHO_CHANNEL_JOIN_SUCCESS,
    /// #64: BANCHO_CHANNEL_JOIN_SUCCESS
    ChannelJoin<'a> { channel_name: CowStr<'a> }
);

packet_struct!(
    PacketId::BANCHO_CHANNEL_INFO,
    /// #65: BANCHO_CHANNEL_INFO
    ChannelInfo<'a> {
        name: CowStr<'a>,
        title: CowStr<'a>,
        player_count: i16,
    }
);

packet_struct!(
    PacketId::BANCHO_CHANNEL_KICK,
    /// #66: BANCHO_CHANNEL_KICK
    ChannelKick<'a> {
        channel_name: CowStr<'a>
    }
);

packet_struct!(
    PacketId::BANCHO_CHANNEL_AUTO_JOIN,
    /// #67: BANCHO_CHANNEL_AUTO_JOIN
    ChannelAutoJoin<'a> {
        name: CowStr<'a>,
        title: CowStr<'a>,
        player_count: i16,
    }
);

packet_struct!(
    PacketId::BANCHO_BEATMAP_INFO_REPLY,
    /// #69: BANCHO_BEATMAP_INFO_REPLY
    /// UNUSED
    BeatmapInfoReply {},
    fn into_packet_data(self) -> Vec<u8> {
        unimplemented!()
    }
);

packet_struct!(
    PacketId::BANCHO_PRIVILEGES,
    /// #71: BANCHO_PRIVILEGES
    BanchoPrivileges { privileges: i32 }
);

packet_struct!(
    PacketId::BANCHO_FRIENDS_LIST,
    /// #72: BANCHO_FRIENDS_LIST
    FriendsList<'a> { friends: &'a [i32] }
);

packet_struct!(
    PacketId::BANCHO_PROTOCOL_VERSION,
    /// #75: BANCHO_PROTOCOL_VERSION
    ProtocolVersion { version: i32 }
);

packet_struct!(
    PacketId::BANCHO_MAIN_MENU_ICON,
    /// #76: BANCHO_MAIN_MENU_ICON
    MainMenuIcon<'a> {
        image_url: CowStr<'a>,
        link_url: CowStr<'a>
    },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(
            Self::ID,
            format!("{}|{}", self.image_url, self.link_url)
        )
    }
);

packet_struct!(
    PacketId::BANCHO_MONITOR,
    /// #80: BANCHO_MONITOR
    /// DEPRECATED
    Monitor {}
);

packet_struct!(
    PacketId::BANCHO_MATCH_PLAYER_SKIPPED,
    /// #81: BANCHO_MATCH_PLAYER_SKIPPED
    MatchPlayerSkipped { slot_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_USER_PRESENCE,
    /// #83: BANCHO_USER_PRESENCE
    UserPresence<'a> {
        user_id: i32,
        username: CowStr<'a>,
        utc_offset: u8,
        country_code: u8,
        bancho_priv: i32,
        longitude: f32,
        latitude: f32,
        rank: i32,
    },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(
            Self::ID,
            self.user_id,
            self.username,
            self.utc_offset + 24,
            self.country_code,
            self.bancho_priv as u8,
            self.longitude,
            self.latitude,
            self.rank
        )
    }
);

packet_struct!(
    PacketId::BANCHO_RESTART,
    /// #86: BANCHO_RESTART
    BanchoRestart { millis: i32 }
);

packet_struct!(
    PacketId::BANCHO_MATCH_INVITE,
    /// #88: BANCHO_MATCH_INVITE
    MatchInvite<'a> {
        welcome: CowStr<'a>,
        match_id: i32,
        match_password: Option<CowStr<'a>>,
    },
    fn into_packet_data(self) -> Vec<u8> {
        let data = format!(
            "{} osump://{}/{}.",
            self.welcome,
            self.match_id,
            self.match_password.unwrap_or(Cow::Borrowed(""))
        );
        packet!(
            Self::ID,
            data
        )
    }
);

packet_struct!(
    PacketId::BANCHO_CHANNEL_INFO_END,
    /// #89: BANCHO_CHANNEL_INFO_END
    ChannelInfoEnd {}
);

packet_struct!(
    PacketId::BANCHO_MATCH_CHANGE_PASSWORD,
    /// #91: BANCHO_MATCH_CHANGE_PASSWORD
    MatchChangePassword<'a> {
        password: CowStr<'a>
    }
);

packet_struct!(
    PacketId::BANCHO_SILENCE_END,
    /// #92: BANCHO_SILENCE_END
    SilenceEnd { duration: i32 }
);

packet_struct!(
    PacketId::BANCHO_USER_SILENCED,
    /// #94: BANCHO_USER_SILENCED
    UserSilenced { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_USER_PRESENCE_SINGLE,
    /// #95: BANCHO_USER_PRESENCE_SINGLE
    UserPresenceSingle { user_id: i32 }
);

packet_struct!(
    PacketId::BANCHO_USER_PRESENCE_BUNDLE,
    /// #96: BANCHO_USER_PRESENCE_BUNDLE
    UserPresenceBundle<'a> { player_ids: &'a [i32] }
);

packet_struct!(
    PacketId::BANCHO_USER_DM_BLOCKED,
    /// #100: BANCHO_USER_DM_BLOCKED
    UserDmBlocked<'a> {
        target: CowStr<'a>
    },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(
            PacketId::BANCHO_USER_DM_BLOCKED,
            "", "", self.target, 0_i32
        )
    }
);

packet_struct!(
    PacketId::BANCHO_TARGET_IS_SILENCED,
    /// #101: BANCHO_TARGET_IS_SILENCED
    TargetSilenced<'a> {
        target: CowStr<'a>
    },
    fn into_packet_data(self) -> Vec<u8> {
        packet!(
            PacketId::BANCHO_USER_DM_BLOCKED,
            "", "", self.target, 0_i32
        )
    }
);

packet_struct!(
    PacketId::BANCHO_VERSION_UPDATE_FORCED,
    /// #102: BANCHO_VERSION_UPDATE_FORCED
    VersionUpdateForced {}
);

packet_struct!(
    PacketId::BANCHO_SWITCH_SERVER,
    /// #103: BANCHO_SWITCH_SERVER
    SwitchServer { time: i32 }
);

packet_struct!(
    PacketId::BANCHO_ACCOUNT_RESTRICTED,
    /// #104: BANCHO_ACCOUNT_RESTRICTED
    AccountRestricted {}
);

packet_struct!(
    PacketId::BANCHO_RTX,
    /// #105: BANCHO_RTX
    /// DEPRECATED
    Rtx<'a> {
        msg: CowStr<'a>
    }
);

packet_struct!(
    PacketId::BANCHO_MATCH_ABORT,
    /// #106: BANCHO_MATCH_ABORT
    MatchAbort {}
);

packet_struct!(
    PacketId::BANCHO_SWITCH_TOURNAMENT_SERVER,
    /// #107: BANCHO_SWITCH_TOURNAMENT_SERVER
    SwitchTournamentServer<'a> {
        ip: CowStr<'a>
    }
);
