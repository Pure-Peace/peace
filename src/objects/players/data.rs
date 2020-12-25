use crate::{constants::PresenceFilter, types::Location};
use chrono::{DateTime, Local};

use super::{
    Player,
    {stats::Stats, status::Status},
};

#[derive(Debug)]
pub struct PlayerData {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub bancho_privileges: i32,
    pub friends: Vec<i32>,
    pub country: String,
    pub ip: String,
    pub address_id: i32,
    pub address_similarity: i32,
    pub only_friend_pm_allowed: bool,
    pub presence_filter: PresenceFilter,
    pub display_city: bool,
    pub osu_version: String,
    pub utc_offset: u8,
    pub location: Location,
    pub stats: Stats,
    pub status: Status,
    pub channels: Vec<String>,
    pub login_time: DateTime<Local>,
    pub login_record_id: i64,
    pub last_active_time: DateTime<Local>,
    pub data_create_time: DateTime<Local>,
}

impl PlayerData {
    pub fn from(p: &Player) -> Self {
        PlayerData {
            id: p.id,
            name: p.name.clone(),
            privileges: p.privileges,
            bancho_privileges: p.bancho_privileges,
            friends: p.friends.clone(),
            country: p.country.clone(),
            ip: p.ip.clone(),
            address_id: p.address_id,
            address_similarity: p.address_similarity,
            only_friend_pm_allowed: p.only_friend_pm_allowed,
            presence_filter: p.presence_filter,
            display_city: p.display_city,
            osu_version: p.osu_version.clone(),
            utc_offset: p.utc_offset,
            location: p.location,
            stats: p.stats.clone(),
            status: p.status.clone(),
            channels: p.channels.iter().map(|s| s.to_string()).collect(),
            login_time: p.login_time,
            login_record_id: p.login_record_id,
            last_active_time: p.last_active_time,
            data_create_time: Local::now(),
        }
    }
}
