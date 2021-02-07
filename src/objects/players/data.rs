use std::convert::TryInto;

use crate::{constants::{GameMode, PresenceFilter}, types::{Location, TokenString}};
use chrono::{DateTime, Local};
use hashbrown::HashMap;

use super::{
    depends::Database,
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
    pub stats_cache: HashMap<GameMode, Stats>,
    pub status: Status,
    pub channels: Vec<String>,
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
            stats_cache: p.stats_cache.clone(),
            status: p.status.clone(),
            channels: p.channels.iter().map(|s| s.to_string()).collect(),
            login_time: p.login_time,
            login_record_id: p.login_record_id,
            token: p.token.clone(),
            last_active_time: p.last_active_time,
            data_create_time: Local::now(),
        }
    }

    /// Returns: Stats, should update cache flag
    #[inline(always)]
    pub async fn get_stats_update(
        &self,
        game_mode: GameMode,
        database: &Database,
    ) -> (Option<Stats>, bool) {
        match self.stats_cache.get(&game_mode) {
            Some(stats) => {
                // Cache expired, get from database
                if Local::now().timestamp() > stats.update_time.timestamp() + 120 {
                    debug!(
                        "Player {}({}) stats cache expired (player-data)! will get new... game mode: {:?}",
                        self.name, self.id, game_mode
                    );
                    return (
                        self.get_stats_from_database(game_mode, database).await,
                        true,
                    );
                };

                debug!(
                    "Player {}({}) stats cache hitted (player-data)! game mode: {:?}",
                    self.name, self.id, game_mode
                );
                // Not expired, return cache
                return (Some(stats.clone()), false);
            }
            None => {
                debug!(
                    "Player {}({}) stats cache not hitted (player-data)! will get new... game mode: {:?}",
                    self.name, self.id, game_mode
                );
                (
                    self.get_stats_from_database(game_mode, database).await,
                    true,
                )
            }
        }
    }

    #[inline(always)]
    pub async fn get_stats_from_database(
        &self,
        game_mode: GameMode,
        database: &Database,
    ) -> Option<Stats> {
        // Build query string
        let (play_mod_name, mode_name) = game_mode.get_table_names();
        let sql = format!(
            r#"SELECT 
                "performance_v1{0}" as "performance_v1",
                "performance_v2{0}" as "performance_v2",
                "accuracy{0}" as "accuracy",
                "total_score{0}" as "total_score",
                "ranked_score{0}" as "ranked_score",
                "playcount{0}" as "playcount",
                "playtime{0}" as "playtime",
                "max_combo{0}" as "max_combo"
            FROM 
                "game_stats"."{1}" 
            WHERE "id" = $1;"#,
            play_mod_name, mode_name
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
                stats
                    .recalculate_rank(&play_mod_name, &mode_name, database)
                    .await;
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
