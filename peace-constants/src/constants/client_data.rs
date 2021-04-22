#![allow(dead_code)]

pub struct ClientInfo {
    pub osu_version: String,
    pub utc_offset: i32,
    pub display_city: bool,
    pub only_friend_pm_allowed: bool,
}

pub struct ClientHashes {
    pub osu_path: String,
    pub adapters: String,
    pub adapters_hash: String,
    pub uninstall_id: String,
    pub disk_id: String,
}
