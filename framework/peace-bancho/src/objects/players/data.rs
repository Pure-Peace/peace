use crate::types::TokenString;
use chrono::{DateTime, Local};
use hashbrown::HashMap;
use peace_constants::{geoip::GeoData, CountryCodes, GameMode, PresenceFilter};
use peace_database::{serde_postgres, Database};

use std::str::FromStr;

use super::{
    Player, PlayerSettings,
    {game_status::GameStatus, stats::Stats},
};

#[derive(Debug)]
pub struct PlayerData {
    pub id: i32,
    pub name: String,
    pub u_name: Option<String>,
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
    pub geo_data: GeoData,
    pub stats: Stats,
    pub settings: PlayerSettings,
    pub stats_cache: HashMap<GameMode, Stats>,
    pub game_status: GameStatus,
    pub away_message: String,
    pub channels: Vec<String>,
    pub spectators: Vec<i32>,
    pub spectating: Option<i32>,
    pub in_lobby: bool,
    pub login_time: DateTime<Local>,
    pub login_record_id: i64,
    pub token: TokenString,
    pub last_active_time: DateTime<Local>,
    pub data_create_time: DateTime<Local>,
}

impl PlayerData {
    pub fn from(p: &Player) -> Self {
        PlayerData {
            id: p.id,
            name: p.name.clone(),
            u_name: p.u_name.clone(),
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
            geo_data: p.geo_data.clone(),
            stats: p.stats.clone(),
            settings: p.settings.clone(),
            stats_cache: p.stats_cache.clone(),
            game_status: p.game_status.clone(),
            away_message: p.away_message.clone(),
            channels: p.channels.iter().map(|s| s.to_string()).collect(),
            spectators: p.spectators.iter().map(|s| *s).collect(),
            spectating: p.spectating.clone(),
            in_lobby: p.in_lobby,
            login_time: p.login_time,
            login_record_id: p.login_record_id,
            token: p.token.clone(),
            last_active_time: p.last_active_time,
            data_create_time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn try_u_name(&self) -> String {
        self.u_name.as_ref().unwrap_or(&self.name).clone()
    }

    #[inline(always)]
    pub fn get_country_code(&self) -> u8 {
        CountryCodes::from_str(&self.country).unwrap_or(CountryCodes::UN) as u8
    }

    #[inline(always)]
    pub async fn get_stats_from_database(
        &self,
        game_mode: &GameMode,
        database: &Database,
    ) -> Option<Stats> {
        // Build query string
        let sql = format!(
            r#"SELECT 
                "pp_v1{0}" as "pp_v1",
                "pp_v2{0}" as "pp_v2",
                "accuracy{0}" as "accuracy",
                "total_hits{0}" as "total_hits",
                "total_score{0}" as "total_score",
                "ranked_score{0}" as "ranked_score",
                "playcount{0}" as "playcount",
                "playtime{0}" as "playtime",
                "max_combo{0}" as "max_combo"
            FROM 
                "game_stats"."{1}" 
            WHERE "id" = $1;"#,
            game_mode.sub_mod_table(),
            game_mode.mode_name()
        );

        // Query from database
        let row = match database.pg.query_first(&sql, &[&self.id]).await {
            Ok(row) => row,
            Err(err) => {
                error!(
                    "Failed to init player {}({})'s play stats from database, error: {:?}",
                    self.name, self.id, err
                );
                return None;
            }
        };

        // Query result into struct
        match serde_postgres::from_row::<Stats>(&row) {
            Ok(mut stats) => {
                debug!(
                    "Success to get player {}({})'s play stats: {:?}",
                    self.name, self.id, stats
                );
                // Calculate rank
                stats.calc_rank_from_database(game_mode, database).await;
                // Done
                return Some(stats);
            }
            Err(err) => {
                error!(
                    "Failed to deserialize player {}({})'s play stats from database, error: {:?}",
                    self.name, self.id, err
                );
                return None;
            }
        };
    }
}
