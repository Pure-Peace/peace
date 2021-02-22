use maxminddb::Reader;
use memmap::Mmap;

use crate::{constants::GeoData, utils};

use super::depends::*;

#[derive(Debug)]
pub struct Player {
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
    pub geo_data: GeoData,
    pub stats: Stats,
    pub stats_cache: HashMap<GameMode, Stats>,
    pub status: Status,
    pub away_message: String,
    pub queue: Mutex<Queue<PacketData>>,
    pub channels: HashSet<String>,
    pub spectators: HashSet<i32>,
    pub spectating: Option<i32>,
    pub login_time: DateTime<Local>,
    pub login_record_id: i64,
    pub token: TokenString,
    pub last_active_time: DateTime<Local>,
}

impl Drop for Player {
    fn drop(&mut self) {
        debug!(
            "Player {}({}) object dropped! token: {}",
            self.name, self.id, self.token
        );
    }
}

impl Player {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    pub async fn from_base(
        base: PlayerBase,
        client_info: ClientInfo,
        ip: String,
        address_id: i32,
        address_similarity: i32,
    ) -> Self {
        let now_time = Local::now();

        Player {
            id: base.id,
            name: base.name,
            privileges: base.privileges,
            bancho_privileges: Player::bancho_privileges(base.privileges),
            friends: vec![base.id],
            country: base.country,
            ip: ip.clone(),
            address_id,
            address_similarity,
            only_friend_pm_allowed: client_info.only_friend_pm_allowed,
            presence_filter: PresenceFilter::None,
            display_city: client_info.display_city,
            osu_version: client_info.osu_version,
            utc_offset: client_info.utc_offset as u8,
            geo_data: GeoData::new(ip),
            stats: Stats::new(),
            stats_cache: HashMap::with_capacity(4),
            status: Status::new(),
            away_message: String::new(),
            queue: Mutex::new(Queue::new()),
            channels: HashSet::new(),
            spectators: HashSet::new(),
            spectating: None,
            login_time: now_time,
            login_record_id: -1,
            token: Uuid::new_v4().to_string(),
            last_active_time: now_time,
        }
    }

    #[inline(always)]
    pub fn update_status(
        &mut self,
        action: Action,
        info: String,
        playing_beatmap_md5: String,
        playing_beatmap_id: i32,
        play_mods_value: u32,
        game_mode: GameMode,
    ) {
        self.status.action = action;
        self.status.info = info;
        self.status.playing_beatmap_md5 = playing_beatmap_md5;
        self.status.playing_beatmap_id = playing_beatmap_id;
        self.status.play_mods.update(play_mods_value);
        self.status.game_mode = game_mode;
        self.status.update_time = Local::now();
    }

    #[inline(always)]
    pub fn update_ip(&mut self, ip_address: String, geo_db: &Option<Reader<Mmap>>) {
        // Update ip
        self.ip = ip_address.clone();
        // Try update geo ip data
        if let Some(geo_db) = geo_db {
            match utils::get_geo_ip_data(&ip_address, geo_db) {
                Ok(geo_data) => self.geo_data = geo_data,
                Err(_) => {}
            }
        }
    }

    #[inline(always)]
    pub fn bancho_privileges(privileges: i32) -> i32 {
        let mut bancho_priv = 0;

        if Privileges::Normal.enough(privileges) {
            // all players have in-game "supporter".
            // this enables stuff like osu!direct,
            // multiplayer in cutting edge, etc.
            bancho_priv |= BanchoPrivileges::Player as i32 | BanchoPrivileges::Supporter as i32
        }

        if Privileges::Mod.enough(privileges) {
            bancho_priv |= BanchoPrivileges::Moderator as i32
        }

        if Privileges::Admin.enough(privileges) {
            bancho_priv |= BanchoPrivileges::Developer as i32
        }

        if Privileges::Dangerous.enough(privileges) {
            bancho_priv |= BanchoPrivileges::Owner as i32
        }

        bancho_priv
    }

    #[inline(always)]
    pub fn update_active(&mut self) {
        self.last_active_time = Local::now();
    }

    pub async fn update_friends(&mut self, database: &Data<Database>) {
        match database
            .pg
            .query(
                r#"SELECT "friend_id" FROM "user"."friends" WHERE "user_id" = $1;"#,
                &[&self.id],
            )
            .await
        {
            Ok(rows) => {
                let mut friends = vec![self.id];
                friends.extend::<Vec<i32>>(rows.iter().map(|row| row.get("friend_id")).collect());
                self.friends = friends;
            }
            Err(err) => error!(
                "Error when update_friends; user: {}({}); err: {:?}",
                self.name, self.id, err
            ),
        };
    }

    /// Create new login record
    pub async fn create_login_record(&mut self, database: &Data<Database>) {
        self.login_record_id = match database
            .pg
            .query_first(
                r#"INSERT INTO "user_records"."login" (
                    "user_id",
                    "address_id",
                    "ip",
                    "version",
                    "similarity"
                 ) VALUES ($1, $2, $3, $4, $5) RETURNING "id";"#,
                &[
                    &self.id,
                    &self.address_id,
                    &self.ip,
                    &self.osu_version,
                    &self.address_similarity,
                ],
            )
            .await
        {
            Ok(row) => row.get("id"),
            Err(err) => {
                error!(
                    "Failed to insert user {}({})'s login record, error: {:?}",
                    self.name, self.id, err
                );
                -1
            }
        };
    }

    /// If user has login record id, record logout time
    pub async fn update_logout_time(&mut self, database: &Database) {
        if self.login_record_id > 0 {
            match database
                .pg
                .execute(
                    r#"UPDATE "user_records"."login" 
                        SET "logout_time" = now() 
                        WHERE "id" = $1;"#,
                    &[&self.login_record_id],
                )
                .await
            {
                Ok(_count) => {
                    self.login_record_id = -1;
                }
                Err(err) => {
                    error!(
                        "Failed to update user {}({})'s logout time, error: {:?}",
                        self.name, self.id, err
                    );
                }
            };
        }
    }

    #[inline(always)]
    pub async fn update_stats(&mut self, database: &Database) {
        match self.stats_cache.get(&self.status.game_mode) {
            Some(stats) => {
                // Cache expired, update from database
                if Local::now().timestamp() > stats.update_time.timestamp() + 120 {
                    debug!(
                        "Player {}({}) stats cache expired! will get new... game mode: {:?}",
                        self.name, self.id, self.status.game_mode
                    );

                    if let Some(stats) = self.get_stats_from_database(database).await {
                        self.stats_cache
                            .insert(self.status.game_mode, stats.clone());
                        self.stats = stats;
                        return;
                    }
                }

                debug!(
                    "Player {}({}) stats cache hitted! game mode: {:?}",
                    self.name, self.id, self.status.game_mode
                );
                // Not expired, return cache
                self.stats = stats.clone();
            }
            // Non-cache, get from database then cache it
            None => {
                debug!(
                    "Player {}({}) stats cache not hitted! will get new... game mode: {:?}",
                    self.name, self.id, self.status.game_mode
                );

                if let Some(stats) = self.get_stats_from_database(database).await {
                    self.stats_cache
                        .insert(self.status.game_mode, stats.clone());
                    self.stats = stats;
                }
            }
        }
    }

    #[inline(always)]
    pub async fn get_stats_from_database(&self, database: &Database) -> Option<Stats> {
        // Build query string
        let (play_mod_name, mode_name) = self.status.game_mode.get_table_names();
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
                    "Failed to get player {}({})'s play stats from database, error: {:?}",
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

    #[inline(always)]
    /// Enqueue a packet into queue, returns the length of queue
    pub async fn enqueue(&self, packet_data: PacketData) -> usize {
        self.queue
            .lock()
            .await
            .queue(packet_data)
            .unwrap_or_else(|_| {
                error!(
                    "Could not enqueue packet to player: {}({})",
                    self.name, self.id
                );
                0
            })
    }

    #[inline(always)]
    pub async fn dequeue(&mut self) -> Option<PacketData> {
        self.queue.lock().await.dequeue()
    }

    #[inline(always)]
    /// Get the queue data as vec, readonly
    pub async fn queue_data(&self) -> Vec<PacketData> {
        self.queue.lock().await.vec().clone()
    }

    #[inline(always)]
    /// Get the queue size
    pub async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }

    #[inline(always)]
    pub async fn queue_peek(&self) -> Option<PacketData> {
        self.queue.lock().await.peek()
    }

    #[inline(always)]
    pub async fn queue_is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }
}
