#![allow(dead_code)]
/// Login replys (i32)
///
/// ```
/// LoginReply {
///     InvalidCredentials        = -1,
///     OutdatedClient            = -2,
///     UserBanned                = -3,
///     MultiaccountDetected      = -4,
///     ServerError               = -5,
///     CuttingEdgeMultiplayer    = -6,
///     AccountPasswordRest       = -7,
///     VerificationRequired      = -8
/// }
///
/// ```
///
pub enum LoginReply {
    InvalidCredentials = -1,
    OutdatedClient = -2,
    UserBanned = -3,
    MultiaccountDetected = -4,
    ServerError = -5,
    CuttingEdgeMultiplayer = -6,
    AccountPasswordRest = -7,
    VerificationRequired = -8,
}


pub mod id {
    /*
        Thanks to gulag
        https://github.com/cmyui/gulag/blob/master/packets.py
    */

    /// Bancho packet ids
    pub const OSU_CHANGE_ACTION:                  u8 = 0;
    pub const OSU_SEND_PUBLIC_MESSAGE:            u8 = 1;
    pub const OSU_LOGOUT:                         u8 = 2;
    pub const OSU_REQUEST_STATUS_UPDATE:          u8 = 3;
    pub const OSU_PING:                           u8 = 4;
    pub const BANCHO_USER_LOGIN_REPLY:            u8 = 5;
    pub const BANCHO_SEND_MESSAGE:                u8 = 7;
    pub const BANCHO_PONG:                        u8 = 8;
    pub const BANCHO_HANDLE_IRC_CHANGE_USERNAME:  u8 = 9;
    pub const BANCHO_HANDLE_IRC_QUIT:             u8 = 10;
    pub const BANCHO_USER_STATS:                  u8 = 11;
    pub const BANCHO_USER_LOGOUT:                 u8 = 12;
    pub const BANCHO_SPECTATOR_JOINED:            u8 = 13;
    pub const BANCHO_SPECTATOR_LEFT:              u8 = 14;
    pub const BANCHO_SPECTATE_FRAMES:             u8 = 15;
    pub const OSU_START_SPECTATING:               u8 = 16;
    pub const OSU_STOP_SPECTATING:                u8 = 17;
    pub const OSU_SPECTATE_FRAMES:                u8 = 18;
    pub const BANCHO_VERSION_UPDATE:              u8 = 19;
    pub const OSU_ERROR_REPORT:                   u8 = 20;
    pub const OSU_CANT_SPECTATE:                  u8 = 21;
    pub const BANCHO_SPECTATOR_CANT_SPECTATE:     u8 = 22;
    pub const BANCHO_GET_ATTENTION:               u8 = 23;
    pub const BANCHO_NOTIFICATION:                u8 = 24;
    pub const OSU_SEND_PRIVATE_MESSAGE:           u8 = 25;
    pub const BANCHO_UPDATE_MATCH:                u8 = 26;
    pub const BANCHO_NEW_MATCH:                   u8 = 27;
    pub const BANCHO_DISBAND_MATCH:                u8 = 28;
    pub const OSU_PART_LOBBY:                     u8 = 29;
    pub const OSU_JOIN_LOBBY:                     u8 = 30;
    pub const OSU_CREATE_MATCH:                   u8 = 31;
    pub const OSU_JOIN_MATCH:                     u8 = 32;
    pub const OSU_PART_MATCH:                     u8 = 33;
    pub const BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS: u8 = 34;
    pub const BANCHO_MATCH_JOIN_SUCCESS:          u8 = 36;
    pub const BANCHO_MATCH_JOIN_FAIL:             u8 = 37;
    pub const OSU_MATCH_CHANGE_SLOT:              u8 = 38;
    pub const OSU_MATCH_READY:                    u8 = 39;
    pub const OSU_MATCH_LOCK:                     u8 = 40;
    pub const OSU_MATCH_CHANGE_SETTINGS:          u8 = 41;
    pub const BANCHO_FELLOW_SPECTATOR_JOINED:     u8 = 42;
    pub const BANCHO_FELLOW_SPECTATOR_LEFT:       u8 = 43;
    pub const OSU_MATCH_START:                    u8 = 44;
    pub const BANCHO_ALL_PLAYERS_LOADED:          u8 = 45;
    pub const BANCHO_MATCH_START:                 u8 = 46;
    pub const OSU_MATCH_SCORE_UPDATE:             u8 = 47;
    pub const BANCHO_MATCH_SCORE_UPDATE:          u8 = 48;
    pub const OSU_MATCH_COMPLETE:                 u8 = 49;
    pub const BANCHO_MATCH_TRANSFER_HOST:         u8 = 50;
    pub const OSU_MATCH_CHANGE_MODS:              u8 = 51;
    pub const OSU_MATCH_LOAD_COMPLETE:            u8 = 52;
    pub const BANCHO_MATCH_ALL_PLAYERS_LOADED:    u8 = 53;
    pub const OSU_MATCH_NO_BEATMAP:               u8 = 54;
    pub const OSU_MATCH_NOT_READY:                u8 = 55;
    pub const OSU_MATCH_FAILED:                   u8 = 56;
    pub const BANCHO_MATCH_PLAYER_FAILED:         u8 = 57;
    pub const BANCHO_MATCH_COMPLETE:              u8 = 58;
    pub const OSU_MATCH_HAS_BEATMAP:              u8 = 59;
    pub const OSU_MATCH_SKIP_REQUEST:             u8 = 60;
    pub const BANCHO_MATCH_SKIP:                  u8 = 61;
    pub const BANCHO_UNAUTHORIZED:                u8 = 62;
    pub const OSU_CHANNEL_JOIN:                   u8 = 63;
    pub const BANCHO_CHANNEL_JOIN_SUCCESS:        u8 = 64;
    pub const BANCHO_CHANNEL_INFO:                u8 = 65;
    pub const BANCHO_CHANNEL_KICK:                u8 = 66;
    pub const BANCHO_CHANNEL_AUTO_JOIN:           u8 = 67;
    pub const OSU_BEATMAP_INFO_REQUEST:           u8 = 68;
    pub const BANCHO_BEATMAP_INFO_REPLY:          u8 = 69;
    pub const OSU_MATCH_TRANSFER_HOST:            u8 = 70;
    pub const BANCHO_PRIVILEGES:                  u8 = 71;
    pub const BANCHO_FRIENDS_LIST:                u8 = 72;
    pub const OSU_FRIEND_ADD:                     u8 = 73;
    pub const OSU_FRIEND_REMOVE:                  u8 = 74;
    pub const BANCHO_PROTOCOL_VERSION:            u8 = 75;
    pub const BANCHO_MAIN_MENU_ICON:              u8 = 76;
    pub const OSU_MATCH_CHANGE_TEAM:              u8 = 77;
    pub const OSU_CHANNEL_PART:                   u8 = 78;
    pub const OSU_RECEIVE_UPDATES:                u8 = 79;
    pub const BANCHO_MONITOR:                     u8 = 80;
    pub const BANCHO_MATCH_PLAYER_SKIPPED:        u8 = 81;
    pub const OSU_SET_AWAY_MESSAGE:               u8 = 82;
    pub const BANCHO_USER_PRESENCE:               u8 = 83;
    pub const OSU_IRC_ONLY:                       u8 = 84;
    pub const OSU_USER_STATS_REQUEST:             u8 = 85;
    pub const BANCHO_RESTART:                     u8 = 86;
    pub const OSU_MATCH_INVITE:                   u8 = 87;
    pub const BANCHO_MATCH_INVITE:                u8 = 88;
    pub const BANCHO_CHANNEL_INFO_END:            u8 = 89;
    pub const OSU_MATCH_CHANGE_PASSWORD:          u8 = 90;
    pub const BANCHO_MATCH_CHANGE_PASSWORD:       u8 = 91;
    pub const BANCHO_SILENCE_END:                 u8 = 92;
    pub const OSU_TOURNAMENT_MATCH_INFO_REQUEST:  u8 = 93;
    pub const BANCHO_USER_SILENCED:               u8 = 94;
    pub const BANCHO_USER_PRESENCE_SINGLE:        u8 = 95;
    pub const BANCHO_USER_PRESENCE_BUNDLE:        u8 = 96;
    pub const OSU_USER_PRESENCE_REQUEST:          u8 = 97;
    pub const OSU_USER_PRESENCE_REQUEST_ALL:      u8 = 98;
    pub const OSU_TOGGLE_BLOCK_NON_FRIEND_DMS:    u8 = 99;
    pub const BANCHO_USER_DM_BLOCKED:             u8 = 100;
    pub const BANCHO_TARGET_IS_SILENCED:          u8 = 101;
    pub const BANCHO_VERSION_UPDATE_FORCED:       u8 = 102;
    pub const BANCHO_SWITCH_SERVER:               u8 = 103;
    pub const BANCHO_ACCOUNT_RESTRICTED:          u8 = 104;
    pub const BANCHO_RTX:                         u8 = 105;
    pub const BANCHO_MATCH_ABORT:                 u8 = 106;
    pub const BANCHO_SWITCH_TOURNAMENT_SERVER:    u8 = 107;
    pub const OSU_TOURNAMENT_JOIN_MATCH_CHANNEL:  u8 = 108;
    pub const OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL: u8 = 109;
}