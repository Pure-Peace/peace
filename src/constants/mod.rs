#![allow(non_camel_case_types)]
mod client_data;
mod common;
mod country;
mod geoip;
mod packets;
mod privileges;
pub mod regexes;

use enum_primitive_derive::Primitive;
use serde::{de::Error, Deserialize, Deserializer};
use strum_macros::EnumIter;

pub use client_data::*;
pub use common::*;
pub use country::*;
pub use geoip::*;
use num_traits::FromPrimitive;
pub use packets::*;
pub use privileges::{BanchoPrivileges, Privileges};

pub const CHEAT_DETECTED_DECREASE_CREDIT: i32 = 200;

#[derive(Debug, Clone)]
pub enum RankStatusInServer {
    NotSubmitted    = -1,
    Pending         = 0,
    Outdated        = 1,
    Ranked          = 2,
    Approved        = 3,
    Qualified       = 4,
    Loved           = 5,
    Unknown,
}

impl RankStatusInServer {
    #[inline(always)]
    pub fn from_api_rank_status(i: i32) -> Self {
        match i {
            -2 => Self::Pending,
            -1 => Self::Pending,
            0 => Self::Pending,
            1 => Self::Ranked,
            2 => Self::Approved,
            3 => Self::Qualified,
            4 => Self::Loved,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubmissionStatus {
    Failed          = 0,
    Passed          = 1,
    PassedAndTop    = 2,
}

impl SubmissionStatus {
    #[inline(always)]
    pub fn val(&self) -> i16 {
        *self as i16
    }
}

#[derive(Debug, Clone)]
pub enum RankStatusInOsuApi {
    Graveyard   = -2,
    Wip         = -1,
    Pending     = 0,
    Ranked      = 1,
    Approved    = 2,
    Qualified   = 3,
    Loved       = 4,
    Unknown,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum ScoreboardType {
    Local   = 0,
    Global  = 1,
    PlayMod = 2,
    Friends = 3,
    Country = 4,
}

impl<'de> Deserialize<'de> for ScoreboardType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let scoreboard_value: u8 = Deserialize::deserialize(deserializer)?;
        match ScoreboardType::parse(scoreboard_value) {
            Some(s) => Ok(s),
            None => Err("invalid scoreboard type value").map_err(D::Error::custom),
        }
    }
}

impl ScoreboardType {
    pub fn parse(value: u8) -> Option<Self> {
        ScoreboardType::from_u8(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
pub enum PresenceFilter {
    // A class to represent the update scope the client wishes to receive
    None    = 0,
    All     = 1,
    Friends = 2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum Action {
    // A class to represent the client's current state
    Idle          = 0,
    Afk           = 1,
    Playing       = 2,
    Editing       = 3,
    Modding       = 4,
    Multiplayer   = 5,
    Watching      = 6,
    Unknown       = 7,
    Testing       = 8,
    Submitting    = 9,
    Paused        = 10,
    Lobby         = 11,
    Multiplaying  = 12,
    Direct        = 13,
}

impl Action {
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive, EnumIter)]
#[repr(u32)]
pub enum PlayMod {
    NoMod         = 0,
    // Down: 1 << 0 ~ 30
    NoFail        = 1,
    Easy          = 2,
    TouchScreen   = 4,
    Hidden        = 8,
    HardRock      = 16,
    SuddenDeath   = 32,
    DoubleTime    = 64,
    Relax         = 128,
    HalfTime      = 256,
    NightCore     = 512,
    FlashLight    = 1024,
    Auto          = 2048,
    SpunOut       = 4096,
    AutoPilot     = 8192,
    Perfect       = 16384,
    Key4          = 32768,
    Key5          = 65536,
    Key6          = 131072,
    Key7          = 262144,
    Key8          = 524288,
    FadeIn        = 1048576,
    Random        = 2097152,
    Cinema        = 4194304,
    Target        = 8388608,
    Key9          = 16777216,
    KeyCoop       = 33554432,
    Key1          = 67108864,
    Key3          = 134217728,
    Key2          = 268435456,
    ScoreV2       = 536870912,
    Mirror        = 1073741824,
    // XXX: needs some modification to work..
    // KEY_MOD = KEY1 | KEY2 | KEY3 | KEY4 | KEY5 | KEY6 | KEY7 | KEY8 | KEY9 | KEYCOOP
    // FREE_MOD_ALLOWED = NOFAIL | EASY | HIDDEN | HARDROCK | \
    //                  SUDDENDEATH | FLASHLIGHT | FADEIN | \
    //                  RELAX | AUTOPILOT | SPUNOUT | KEY_MOD
    // SCORE_INCREASE_MODS = HIDDEN | HARDROCK | DOUBLETIME | FLASHLIGHT | FADEIN

    // DoubleTime | NightCore | HalfTime
    // SPEED_CHANGING = 832
}

impl PlayMod {
    #[inline(always)]
    pub fn val(&self) -> u32 {
        *self as u32
    }

    #[inline(always)]
    pub fn contains(&self, value: u32) -> bool {
        (value & self.val()) > 0
    }

    #[inline(always)]
    pub fn not_contains(&self, value: u32) -> bool {
        (value & self.val()) == 0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive, Hash)]
#[repr(u8)]
pub enum GameMode {
    Std       = 0,
    Taiko     = 1,
    Catch     = 2,
    Mania     = 3,

    Std_rx    = 4,
    Taiko_rx  = 5,
    Catch_rx  = 6,
    // Mania_rx  = 7, but not exists
    Std_ap    = 8,

    Std_scv2  = 12,
}

impl<'de> Deserialize<'de> for GameMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let game_mode_value: u8 = Deserialize::deserialize(deserializer)?;
        match GameMode::parse(game_mode_value) {
            Some(s) => Ok(s),
            None => Err("invalid game mode value").map_err(D::Error::custom),
        }
    }
}

impl GameMode {
    #[inline(always)]
    pub fn parse(game_mode_u8: u8) -> Option<Self> {
        GameMode::from_u8(game_mode_u8)
    }

    #[inline(always)]
    pub fn pp_is_best(&self) -> bool {
        match self {
            Self::Std_rx => true,
            Self::Std_ap => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn parse_with_playmod(game_mode_u8: u8, playmod_list: &Vec<PlayMod>) -> Option<Self> {
        let game_mode_u8 = GameMode::value_with_playmod(game_mode_u8, playmod_list);
        GameMode::from_u8(game_mode_u8)
    }

    #[inline(always)]
    pub fn update_with_playmod(&mut self, playmod_list: &Vec<PlayMod>) {
        let game_mode_u8 = GameMode::value_with_playmod(*self as u8, playmod_list);
        if let Some(game_mode) = GameMode::from_u8(game_mode_u8) {
            *self = game_mode;
        }
    }

    #[inline(always)]
    pub fn value_with_playmod(mut game_mode_u8: u8, playmod_list: &Vec<PlayMod>) -> u8 {
        // !More detailed game mod but:
        //
        // 1. Mania have not relax
        // 2. only std have autopilot
        // 3. relax and autopilot cannot coexist
        //
        if game_mode_u8 < 4 && game_mode_u8 != 3 && playmod_list.contains(&PlayMod::Relax) {
            game_mode_u8 += 4;
        } else if game_mode_u8 == 0 {
            if playmod_list.contains(&PlayMod::AutoPilot) {
                game_mode_u8 += 8;
            } else if playmod_list.contains(&PlayMod::ScoreV2) {
                game_mode_u8 += 12;
            }
        }
        game_mode_u8
    }

    #[inline(always)]
    pub fn raw_value(&self) -> u8 {
        self.val() % 4
    }

    #[inline(always)]
    pub fn val(&self) -> u8 {
        *self as u8
    }

    #[inline(always)]
    pub fn is_rx(&self) -> bool {
        let self_value = self.val();
        self_value > 3 && self_value < 8
    }

    #[inline(always)]
    pub fn is_ap(&self) -> bool {
        self.val() == 8
    }

    #[inline(always)]
    pub fn is_scv2(&self) -> bool {
        self.val() == 12
    }

    #[inline(always)]
    pub fn is_vn(&self) -> bool {
        self.val() < 4
    }

    #[inline(always)]
    /// Get lowercase name
    ///
    /// ```
    /// GameMode::Std_rx -> "std_rx"
    /// ```
    pub fn full_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    #[inline(always)]
    /// Get mode name
    ///
    /// ```
    /// GameMode::Std_rx -> "std"
    /// ```
    pub fn mode_name(&self) -> String {
        self.full_name().split('_').collect::<Vec<&str>>()[0].to_string()
    }

    #[inline(always)]
    /// Get sub mod name
    ///
    /// ```
    /// GameMode::Std_rx -> "rx"
    /// ```
    pub fn sub_mod(&self) -> &str {
        match self.val() {
            value if value == 8 => "ap",
            value if value == 12 => "scv2",
            value if value > 3 && value < 8 => "rx",
            _ => "",
        }
    }

    #[inline(always)]
    /// Get mode name (database fields with "_")
    ///
    /// ```
    /// GameMode::Std_rx -> "_rx"
    /// ```
    pub fn sub_mod_table(&self) -> &str {
        match self.val() {
            value if value == 8 => "_ap",
            value if value == 12 => "_scv2",
            value if value > 3 && value < 8 => "_rx",
            _ => "",
        }
    }
}
