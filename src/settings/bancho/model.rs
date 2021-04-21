use serde::{Deserialize, Serialize};

use crate::constants::GameMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanchoConfigData {
    pub server_info: ServerInfo,
    pub server: Server,
    pub online_user_limit: OnlineUserLimit,
    pub menu_icon: MenuIcon,
    pub maintenance: Maintenance,
    pub mutiaccounts: MutiAccounts,
    pub auto_ban: AutoBan,
    pub in_game_registration: InGameRegistration,
    pub login: Login,
    pub client_check: ClientCheck,
    pub message: Message,
    pub client_update: ClientUpdate,
    pub beatmaps: Beatmaps,
    pub session_recycle: SessionRecycle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub front_url: String,
    pub name: String,
    pub owner: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub osu_api_keys: Vec<String>,
    pub all_players_have_supporter: bool,
    pub free_direct: bool,
    pub display_clan_name: bool,
    pub sensitive_words: Vec<String>,
    pub seasonal_backgrounds: Option<Vec<String>>,
    pub ip_blacklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineUserLimit {
    pub enabled: bool,
    pub online_max: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuIcon {
    pub enabled: bool,
    pub image_url: String,
    pub click_url: String,
}

impl MenuIcon {
    #[inline(always)]
    pub fn get(&self) -> Option<String> {
        if self.enabled {
            Some(format!("{}|{}", self.image_url, self.click_url))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintenance {
    pub enabled: bool,
    pub notification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutiAccounts {
    pub enabled: bool,
    pub max_accounts: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBan {
    pub enabled: bool,
    pub id_whitelist: Vec<i32>,
    pub std: Option<i32>,
    pub taiko: Option<i32>,
    pub catch: Option<i32>,
    pub mania: Option<i32>,
    pub std_rx: Option<i32>,
    pub std_ap: Option<i32>,
    pub taiko_rx: Option<i32>,
    pub catch_rx: Option<i32>,
}

impl AutoBan {
    #[inline(always)]
    pub fn pp(&self, mode: &GameMode) -> Option<i32> {
        match mode {
            GameMode::Std => self.std,
            GameMode::Taiko => self.taiko,
            GameMode::Catch => self.catch,
            GameMode::Mania => self.mania,
            GameMode::Std_rx => self.std_rx,
            GameMode::Taiko_rx => self.taiko_rx,
            GameMode::Catch_rx => self.catch_rx,
            GameMode::Std_ap => self.std_ap,
            GameMode::Std_scv2 => self.std,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InGameRegistration {
    pub enabled: bool,
    pub disallowed_ip: Vec<String>,
    pub disallowed_emails: Vec<String>,
    pub disallowed_usernames: Vec<String>,
    pub disallowed_passwords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    pub enabled: bool,
    pub notifications: Vec<String>,
    pub retry_max: i32,
    pub retry_expire: i32,
    pub disallowed_ip: Vec<String>,
    pub disallowed_id: Vec<i32>,
    pub disallowed_usernames: Vec<String>,
    pub disallowed_hardware_hashes: Vec<String>,
    pub disallowed_disk_hashes: Vec<String>,
    pub disallowed_adapters_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCheck {
    pub enabled: bool,
    pub only_whitelist: bool,
    pub id_whitelist: Vec<i32>,
    pub client_whitelist: Vec<String>,
    pub client_blacklist: Vec<String>,
    pub min_version: Option<i32>,
    pub max_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub frequency_limit_enabled: bool,
    pub max_per_minutes: i32,
    pub base_mute_time: i32,
    pub max_length: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUpdate {
    pub enabled: bool,
    pub cache_expires: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beatmaps {
    pub cache_expires: i64,
    pub loved_give_pp: bool,
    pub unranked_give_pp: bool,
    pub all_not_submitted: bool,
    pub all_have_scoreboard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecycle {
    pub check_interval: i32,
    pub session_timeout: i64,
}
