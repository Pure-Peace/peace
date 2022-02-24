use {
    enum_primitive_derive::Primitive,
    num_traits::FromPrimitive,
    serde::{Deserialize, Deserializer},
    strum::IntoEnumIterator,
    strum_macros::EnumIter,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive, EnumIter)]
#[repr(u32)]
pub enum PlayMod {
    NoMod = 0,
    // Down: 1 << 0 ~ 30
    NoFail = 1,
    Easy = 2,
    TouchScreen = 4,
    Hidden = 8,
    HardRock = 16,
    SuddenDeath = 32,
    DoubleTime = 64,
    Relax = 128,
    HalfTime = 256,
    NightCore = 512,
    FlashLight = 1024,
    Auto = 2048,
    SpunOut = 4096,
    AutoPilot = 8192,
    Perfect = 16384,
    Key4 = 32768,
    Key5 = 65536,
    Key6 = 131072,
    Key7 = 262144,
    Key8 = 524288,
    FadeIn = 1048576,
    Random = 2097152,
    Cinema = 4194304,
    Target = 8388608,
    Key9 = 16777216,
    KeyCoop = 33554432,
    Key1 = 67108864,
    Key3 = 134217728,
    Key2 = 268435456,
    ScoreV2 = 536870912,
    Mirror = 1073741824,
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
    #[inline]
    pub fn val(&self) -> u32 {
        *self as u32
    }

    #[inline]
    pub fn contains(&self, value: u32) -> bool {
        (value & self.val()) > 0
    }

    #[inline]
    pub fn not_contains(&self, value: u32) -> bool {
        (value & self.val()) == 0
    }
}

#[derive(Debug, Clone)]
pub struct PlayMods {
    pub value: u32,
    pub list: Vec<PlayMod>,
}

impl<'de> Deserialize<'de> for PlayMods {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let play_mods_value: u32 = Deserialize::deserialize(deserializer)?;
        Ok(PlayMods::parse(play_mods_value))
    }
}

impl PlayMods {
    #[inline]
    /// Initial playmods with NoMod
    pub fn new() -> Self {
        PlayMods {
            value: 0,
            list: vec![PlayMod::NoMod],
        }
    }

    #[inline]
    /// Initial playmods with a single playmod
    pub fn with(playmod: PlayMod) -> Self {
        PlayMods {
            value: playmod as u32,
            list: vec![playmod],
        }
    }

    #[inline]
    pub fn include(&self, play_mod: &PlayMod) -> bool {
        self.list.contains(play_mod)
    }

    #[inline]
    pub fn parse(play_mods_value: u32) -> Self {
        PlayMods {
            value: play_mods_value,
            list: PlayMods::get_mods(play_mods_value),
        }
    }

    #[inline]
    pub fn update(&mut self, play_mods_value: u32) {
        self.value = play_mods_value;
        self.list = self.mods();
    }

    #[inline]
    pub fn get_mods(value: u32) -> Vec<PlayMod> {
        match PlayMod::from_u32(value) {
            Some(play_mod) => vec![play_mod],
            None => PlayMod::iter()
                .filter(|play_mod| play_mod.contains(value))
                .collect(),
        }
    }

    #[inline]
    pub fn mods(&self) -> Vec<PlayMod> {
        PlayMods::get_mods(self.value)
    }
}
