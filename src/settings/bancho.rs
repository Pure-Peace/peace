use crate::database::Database;
use chrono::{DateTime, Utc};
use tokio_pg_mapper::FromTokioPostgresRow;
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
    pub all_players_have_supporter: bool,
}

impl BanchoConfig {
    #[inline(always)]
    /// Initial bancho config from database
    pub async fn from_database(database: &Database) -> Option<BanchoConfig> {
        let row = database
            .pg
            .query_first(
                r#"SELECT * FROM "bancho"."config" WHERE "enabled" = TRUE;"#,
                &[],
            )
            .await;
        if row.is_err() {
            error!(
                "Failed to get bancho config from database! error: {:?}",
                row
            );
            return None;
        }

        let row = row.unwrap();
        let result = BanchoConfig::from_row(row);
        if result.is_err() {
            error!(
                "Failed to deserialize bancho config from pg-row! error: {:?}",
                result
            );
            return None;
        };

        Some(result.unwrap())
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
        info!("New BanchoConfig ({}) updated in {:?}; update time: {}", self.name, start.elapsed(), self.update_time);
        true
    }
}

mod my_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[inline(always)]
fn default_time() -> DateTime<Utc> {
    Utc::now()
}
