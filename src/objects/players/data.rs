use chrono::{DateTime, Local};

use super::Player;

#[derive(Debug)]
pub struct PlayerData {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
    pub osu_version: String,
    pub utc_offset: u8,
    pub login_time: DateTime<Local>,
    pub last_active_time: DateTime<Local>,
}

impl PlayerData {
    pub fn from(player: &Player) -> Self {
        PlayerData {
            id: player.id,
            name: player.name.clone(),
            privileges: player.privileges,
            country: player.country.clone(),
            osu_version: player.osu_version.clone(),
            utc_offset: player.utc_offset,
            login_time: player.login_time,
            last_active_time: player.last_active_time,
        }
    }
}
