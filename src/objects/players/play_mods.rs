use serde::Deserializer;

use super::depends::*;

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
    #[inline(always)]
    pub fn new(playmod: PlayMod) -> Self {
        PlayMods {
            value: playmod as u32,
            list: vec![playmod],
        }
    }

    #[inline(always)]
    pub fn include(&self, play_mod: &PlayMod) -> bool {
        self.list.contains(play_mod)
    }

    #[inline(always)]
    pub fn parse(play_mods_value: u32) -> Self {
        PlayMods {
            value: play_mods_value,
            list: PlayMods::get_mods(play_mods_value),
        }
    }

    #[inline(always)]
    pub fn update(&mut self, play_mods_value: u32) {
        self.value = play_mods_value;
        self.list = self.mods();
    }

    #[inline(always)]
    pub fn get_mods(value: u32) -> Vec<PlayMod> {
        match PlayMod::from_u32(value) {
            Some(play_mod) => vec![play_mod],
            None => PlayMod::iter()
                .filter(|play_mod| play_mod.contains(value))
                .collect(),
        }
    }

    #[inline(always)]
    pub fn mods(&self) -> Vec<PlayMod> {
        PlayMods::get_mods(self.value)
    }
}
