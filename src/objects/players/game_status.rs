use super::{depends::*, PlayMods};

#[derive(Debug, Clone)]
pub struct GameStatus {
    pub action: Action,
    pub info: String,
    pub beatmap_id: i32,
    pub beatmap_md5: String,
    pub mods: PlayMods,
    pub mode: GameMode,
    pub update_time: DateTime<Local>,
}

impl GameStatus {
    pub fn new() -> Self {
        Self {
            action: Action::Idle,
            info: String::new(),
            beatmap_id: 0,
            beatmap_md5: String::new(),
            mods: PlayMods::new(PlayMod::NoMod),
            mode: GameMode::Std,
            update_time: Local::now(),
        }
    }
}
