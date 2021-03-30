use crate::{database::Database, utils};
use chrono::{DateTime, Utc};
use tokio_pg_mapper_derive::PostgresMapper;

#[pg_mapper(table = "bancho.config")]
#[derive(Clone, Debug, PostgresMapper)]
/// Bancho config
pub struct BanchoConfig {
    pub name: String,
    pub update_time: DateTime<Utc>,
    pub osu_api_keys: Vec<String>,
    pub free_direct: bool,
    pub ip_blacklist: Vec<String>,
    pub display_clan_name: bool,
    pub sensitive_words: Vec<String>,
    pub menu_icon: Option<String>,
    pub seasonal_backgrounds: Option<Vec<String>>,

    pub server_front_url: String,
    pub server_name: String,
    pub server_owner: String,
    pub server_email: String,

    pub client_check: bool,
    pub client_only_whitelist: bool,
    pub client_whitelist: Vec<String>,
    pub client_blacklist: Vec<String>,
    pub client_min_version: Option<i32>,
    pub client_max_version: Option<i32>,

    pub beatmaps_loved_give_pp: bool,
    pub beatmaps_unranked_give_pp: bool,

    pub maintenance_enabled: bool,
    pub maintenance_notification: String,

    pub login_enabled: bool,
    pub login_notifications: Vec<String>,

    pub login_retry_max_count: i32,
    pub login_retry_expire_seconds: i32,

    pub timeout_player_session: i64,
    pub timeout_beatmap_cache: i64,
    pub timeout_osu_updates_cache: i64,

    pub online_users_limit: bool,
    pub online_users_max: i32,

    pub message_frequency_limit: bool,
    pub message_per_minutes_max: i32,
    pub message_base_limit_seconds: i64,
    pub message_length_max: Option<i32>,

    pub muti_accounts_allowed: bool,
    pub muti_accounts_max: i32,

    pub auto_ban_enabled: bool,
    pub auto_ban_whitelist: Vec<i32>,
    pub auto_ban_pp_std: Option<i32>,
    pub auto_ban_pp_taiko: Option<i32>,
    pub auto_ban_pp_catch: Option<i32>,
    pub auto_ban_pp_mania: Option<i32>,
    pub auto_ban_pp_rx_std: Option<i32>,
    pub auto_ban_pp_rx_taiko: Option<i32>,
    pub auto_ban_pp_rx_catch: Option<i32>,
    pub auto_ban_pp_ap_std: Option<i32>,

    pub registration_enabled: bool,
    pub registration_disallowed_ip: Vec<String>,
    pub registration_disallowed_emails: Vec<String>,
    pub registration_disallowed_usernames: Vec<String>,
    pub registration_disallowed_passwords: Vec<String>,

    pub login_disallowed_ip: Vec<String>,
    pub login_disallowed_id: Vec<i32>,
    pub login_disallowed_usernames: Vec<String>,
    pub login_disallowed_hardware_hashes: Vec<String>,
    pub login_disallowed_disk_hashes: Vec<String>,
    pub login_disallowed_adapters_hashes: Vec<String>,

    pub all_beatmaps_not_submitted: bool,
    pub all_beatmaps_have_scoreboard: bool,
    pub all_players_have_supporter: bool,
    pub client_update_enabled: bool,
    pub client_update_expires: i32,
    pub session_recycle_check_interval: i32,
}

impl BanchoConfig {
    #[inline(always)]
    /// Initial bancho config from database
    pub async fn from_database(database: &Database) -> Option<BanchoConfig> {
        utils::struct_from_database("bancho", "config", "enabled", "*", &true, database).await
    }

    #[inline(always)]
    /// Update bancho config from database
    pub async fn update(&mut self, database: &Database) -> bool {
        let start = std::time::Instant::now();
        let new = BanchoConfig::from_database(database).await;
        if new.is_none() {
            error!("BanchoConfig update failed.");
            return false;
        };
        *self = new.unwrap();
        info!(
            "New BanchoConfig ({}) updated in {:?}; update time: {}",
            self.name,
            start.elapsed(),
            self.update_time
        );
        true
    }
}
