#![allow(non_camel_case_types)]
mod client_data;
mod common;
mod country;
mod geoip;
mod packets;
mod privileges;

use enum_primitive_derive::Primitive;
use strum_macros::EnumIter;

pub use client_data::*;
pub use common::*;
pub use country::*;
pub use geoip::*;
pub use packets::*;
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
    pub fn val(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    pub fn contains(self, value: u32) -> bool {
        (value & self as u32) > 0
    }

    #[inline(always)]
    pub fn not_contains(self, value: u32) -> bool {
        (value & self as u32) == 0
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
}

impl GameMode {
    #[inline(always)]
    pub fn val(self) -> u8 {
        (self as u8) % 4
    }

    #[inline(always)]
    pub fn is_rx(self) -> bool {
        let self_value = self as u8;
        self_value > 3 && self_value < 8
    }

    #[inline(always)]
    pub fn is_ap(self) -> bool {
        (self as u8) == 8
    }

    #[inline(always)]
    pub fn is_vn(self) -> bool {
        (self as u8) < 4
    }

    #[inline(always)]
    pub fn full_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    #[inline(always)]
    pub fn mode_name(&self) -> String {
        self.full_name().split('_').collect::<Vec<&str>>()[0].to_string()
    }

    #[inline(always)]
    pub fn play_mod_name(self) -> String {
        match self as u8 {
            value if value == 8 => String::from("ap"),
            value if value > 3 && value < 8 => String::from("rx"),
            _ => String::new(),
        }
    }

    #[inline(always)]
    pub fn play_mod_name_table(self) -> String {
        match self.play_mod_name() {
            mut n if n != String::new() => {
                n.insert(0, '_');
                n
            }
            n => n,
        }
    }

    #[inline(always)]
    pub fn get_table_names(self) -> (String, String) {
        (self.play_mod_name_table(), self.mode_name())
    }
}
