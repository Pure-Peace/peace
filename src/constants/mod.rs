#![allow(non_camel_case_types)]
mod client_data;
mod packets;
mod peace;
mod privileges;

use enum_primitive_derive::Primitive;

pub use client_data::*;
pub use packets::*;
pub use peace::*;
pub use privileges::{BanchoPrivileges, Privileges};

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
    pub fn val(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(i32)]
pub enum PlayMods {
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
    SPEED_CHANGING = 832 
}

impl PlayMods {
    pub fn val(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum GameMode {
    Std   = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,

    Std_rx   = 4,
    Taiko_rx = 5,
    Mania_rx = 6,

    Std_ap   = 7,
}

impl GameMode {
    pub fn val(self) -> u8 {
        match self {
            GameMode::Std_ap => GameMode::Std as u8,
            _ => {
                (self as u8) % 4
            }
        }
    }
}
