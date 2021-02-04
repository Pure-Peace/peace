#![allow(dead_code)]
#![allow(non_camel_case_types)]

use enum_primitive_derive::Primitive;

pub trait LoginReply {
    fn val(self) -> i32;
}
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
pub enum LoginFailed {
    InvalidCredentials        = -1,
    OutdatedClient            = -2,
    UserBanned                = -3,
    MultiaccountDetected      = -4,
    ServerError               = -5,
    CuttingEdgeMultiplayer    = -6,
    AccountPasswordRest       = -7,
    VerificationRequired      = -8,
}

impl LoginReply for LoginFailed {
    fn val(self) -> i32 {
        self as i32
    }
}

pub enum LoginSuccess {
    Verified(i32),
}

impl LoginReply for LoginSuccess {
    fn val(self) -> i32 {
        match self {
            LoginSuccess::Verified(user_id) => user_id,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum id {
    /*
        Thanks to gulag
        https://github.com/cmyui/gulag/blob/master/packets.py
    */
    /// Bancho packet ids
    OSU_USER_CHANGE_ACTION                  = 0,
    OSU_SEND_PUBLIC_MESSAGE                 = 1,
    OSU_USER_LOGOUT                         = 2,
    OSU_REQUEST_STATUS_UPDATE               = 3,
    OSU_PING                                = 4,
    BANCHO_USER_LOGIN_REPLY                 = 5,
    BANCHO_SEND_MESSAGE                     = 7,
    BANCHO_PONG                             = 8,
    BANCHO_HANDLE_IRC_CHANGE_USERNAME       = 9,
    BANCHO_HANDLE_IRC_QUIT                  = 10,
    BANCHO_USER_STATS                       = 11,
    BANCHO_USER_LOGOUT                      = 12,
    BANCHO_SPECTATOR_JOINED                 = 13,
    BANCHO_SPECTATOR_LEFT                   = 14,
    BANCHO_SPECTATE_FRAMES                  = 15,
    OSU_START_SPECTATING                    = 16,
    OSU_STOP_SPECTATING                     = 17,
    OSU_SPECTATE_FRAMES                     = 18,
    BANCHO_VERSION_UPDATE                   = 19,
    OSU_ERROR_REPORT                        = 20,
    OSU_CANT_SPECTATE                       = 21,
    BANCHO_SPECTATOR_CANT_SPECTATE          = 22,
    BANCHO_GET_ATTENTION                    = 23,
    BANCHO_NOTIFICATION                     = 24,
    OSU_SEND_PRIVATE_MESSAGE                = 25,
    BANCHO_UPDATE_MATCH                     = 26,
    BANCHO_NEW_MATCH                        = 27,
    BANCHO_DISBAND_MATCH                    = 28,
    OSU_PART_LOBBY                          = 29,
    OSU_JOIN_LOBBY                          = 30,
    OSU_CREATE_MATCH                        = 31,
    OSU_JOIN_MATCH                          = 32,
    OSU_PART_MATCH                          = 33,
    BANCHO_TOGGLE_BLOCK_NON_FRIEND_DMS      = 34,
    BANCHO_MATCH_JOIN_SUCCESS               = 36,
    BANCHO_MATCH_JOIN_FAIL                  = 37,
    OSU_MATCH_CHANGE_SLOT                   = 38,
    OSU_MATCH_READY                         = 39,
    OSU_MATCH_LOCK                          = 40,
    OSU_MATCH_CHANGE_SETTINGS               = 41,
    BANCHO_FELLOW_SPECTATOR_JOINED          = 42,
    BANCHO_FELLOW_SPECTATOR_LEFT            = 43,
    OSU_MATCH_START                         = 44,
    BANCHO_ALL_PLAYERS_LOADED               = 45,
    BANCHO_MATCH_START                      = 46,
    OSU_MATCH_SCORE_UPDATE                  = 47,
    BANCHO_MATCH_SCORE_UPDATE               = 48,
    OSU_MATCH_COMPLETE                      = 49,
    BANCHO_MATCH_TRANSFER_HOST              = 50,
    OSU_MATCH_CHANGE_MODS                   = 51,
    OSU_MATCH_LOAD_COMPLETE                 = 52,
    BANCHO_MATCH_ALL_PLAYERS_LOADED         = 53,
    OSU_MATCH_NO_BEATMAP                    = 54,
    OSU_MATCH_NOT_READY                     = 55,
    OSU_MATCH_FAILED                        = 56,
    BANCHO_MATCH_PLAYER_FAILED              = 57,
    BANCHO_MATCH_COMPLETE                   = 58,
    OSU_MATCH_HAS_BEATMAP                   = 59,
    OSU_MATCH_SKIP_REQUEST                  = 60,
    BANCHO_MATCH_SKIP                       = 61,
    BANCHO_UNAUTHORIZED                     = 62,
    OSU_USER_CHANNEL_JOIN                   = 63,
    BANCHO_CHANNEL_JOIN_SUCCESS             = 64,
    BANCHO_CHANNEL_INFO                     = 65,
    BANCHO_CHANNEL_KICK                     = 66,
    BANCHO_CHANNEL_AUTO_JOIN                = 67,
    OSU_BEATMAP_INFO_REQUEST                = 68,
    BANCHO_BEATMAP_INFO_REPLY               = 69,
    OSU_MATCH_TRANSFER_HOST                 = 70,
    BANCHO_PRIVILEGES                       = 71,
    BANCHO_FRIENDS_LIST                     = 72,
    OSU_USER_FRIEND_ADD                     = 73,
    OSU_USER_FRIEND_REMOVE                  = 74,
    BANCHO_PROTOCOL_VERSION                 = 75,
    BANCHO_MAIN_MENU_ICON                   = 76,
    OSU_MATCH_CHANGE_TEAM                   = 77,
    OSU_USER_CHANNEL_PART                   = 78,
    OSU_USER_RECEIVE_UPDATES                = 79,
    BANCHO_MONITOR                          = 80,
    BANCHO_MATCH_PLAYER_SKIPPED             = 81,
    OSU_SET_AWAY_MESSAGE                    = 82,
    BANCHO_USER_PRESENCE                    = 83,
    OSU_IRC_ONLY                            = 84,
    OSU_USER_STATS_REQUEST                  = 85,
    BANCHO_RESTART                          = 86,
    OSU_MATCH_INVITE                        = 87,
    BANCHO_MATCH_INVITE                     = 88,
    BANCHO_CHANNEL_INFO_END                 = 89,
    OSU_MATCH_CHANGE_PASSWORD               = 90,
    BANCHO_MATCH_CHANGE_PASSWORD            = 91,
    BANCHO_SILENCE_END                      = 92,
    OSU_TOURNAMENT_MATCH_INFO_REQUEST       = 93,
    BANCHO_USER_SILENCED                    = 94,
    BANCHO_USER_PRESENCE_SINGLE             = 95,
    BANCHO_USER_PRESENCE_BUNDLE             = 96,
    OSU_USER_PRESENCE_REQUEST               = 97,
    OSU_USER_PRESENCE_REQUEST_ALL           = 98,
    OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS    = 99,
    BANCHO_USER_DM_BLOCKED                  = 100,
    BANCHO_TARGET_IS_SILENCED               = 101,
    BANCHO_VERSION_UPDATE_FORCED            = 102,
    BANCHO_SWITCH_SERVER                    = 103,
    BANCHO_ACCOUNT_RESTRICTED               = 104,
    BANCHO_RTX                              = 105,
    BANCHO_MATCH_ABORT                      = 106,
    BANCHO_SWITCH_TOURNAMENT_SERVER         = 107,
    OSU_TOURNAMENT_JOIN_MATCH_CHANNEL       = 108,
    OSU_TOURNAMENT_LEAVE_MATCH_CHANNEL      = 109,
    OSU_UNKNOWN_PACKET                      = 255,
}
