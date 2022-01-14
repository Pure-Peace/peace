#![allow(non_camel_case_types)]

use {
    crate::PlayMod,
    enum_primitive_derive::Primitive,
    num_traits::FromPrimitive,
    serde::{de::Error, Deserialize, Deserializer},
};

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
    /// ```rust,ignore
    /// GameMode::Std_rx -> "std_rx"
    /// ```
    pub fn full_name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    #[inline(always)]
    /// Get mode name
    ///
    /// ```rust,ignore
    /// GameMode::Std_rx -> "std"
    /// ```
    pub fn mode_name(&self) -> String {
        self.full_name().split('_').collect::<Vec<&str>>()[0].to_string()
    }

    #[inline(always)]
    /// Get sub mod name
    ///
    /// ```rust,ignore
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
    /// ```rust,ignore
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
