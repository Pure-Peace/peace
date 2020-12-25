use super::{depends::*, PlayMods};

#[derive(Debug, Clone)]
pub struct Status {
    pub action: Action,
    pub info: String,
    pub playing_beatmap_id: i32,
    pub playing_beatmap_md5: String,
    pub play_mods: PlayMods,
    pub game_mode: GameMode,
    pub update_time: DateTime<Local>,
}

impl Status {
    pub fn new() -> Self {
        Status {
            action: Action::Idle,
            info: String::new(),
            playing_beatmap_id: 0,
            playing_beatmap_md5: String::new(),
            play_mods: PlayMods::new(PlayMod::NoMod),
            game_mode: GameMode::Std,
            update_time: Local::now(),
        }
    }
}
