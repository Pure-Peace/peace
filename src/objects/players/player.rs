use maxminddb::Reader;
use memmap::Mmap;

use crate::objects::{PlayerInfo, PlayerSettings};
use derivative::Derivative;
use serde_json::json;
use std::str::FromStr;

use crate::{
    constants::{CountryCodes, GeoData, CHEAT_DETECTED_DECREASE_CREDIT},
    packets,
    types::Argon2Cache,
    utils,
};

use super::{depends::*, PlayMods};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Player {
    pub id: i32,
    pub name: String,
    #[derivative(Debug = "ignore")]
    _password: String,
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
    pub info: PlayerInfo,
    pub settings: PlayerSettings,
    pub stats: Stats,
    pub stats_cache: HashMap<GameMode, Stats>,
    pub status: Status,
    pub away_message: String,
    pub flag_cache: HashMap<String, Option<String>>,
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

    pub async fn create(
        base: PlayerBase,
        info: PlayerInfo,
        settings: PlayerSettings,
        client_info: ClientInfo,
        ip: String,
        address_id: i32,
        address_similarity: i32,
    ) -> Self {
        let now_time = Local::now();

        Player {
            id: base.id,
            name: base.name,
            _password: base.password,
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
            info,
            settings,
            stats: Stats::new(),
            stats_cache: HashMap::with_capacity(4),
            status: Status::new(),
            away_message: String::new(),
            flag_cache: HashMap::new(),
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
    pub async fn recalculate_pp_acc(&mut self, score_table: &str, database: &Database) {
        // Get bp
        let score_set = match database
            .pg
            .query(
                &format!(
                    r#"SELECT s.pp_v2, s.accuracy FROM game_scores.{} s 
                INNER JOIN beatmaps.maps m ON s.map_md5 = m.md5 
                WHERE s.user_id = $1 
                AND s.status = 2 
                AND s.pp_v2 IS NOT NULL
                AND m.rank_status IN (1, 2)
                ORDER BY s.pp_v2 DESC
                LIMIT 100;"#,
                    score_table
                ),
                &[&self.id],
            )
            .await
        {
            Ok(rows) => rows,
            Err(err) => {
                error!(
                    "[recalculate_pp_acc] Failed to get Player({})'s score set, err: {:?}",
                    self.id, err
                );
                return;
            }
        };
        let score_count = score_set.len();

        // Calc acc from bp
        self.stats.accuracy = if score_count == 1 {
            score_set[0]
                .try_get::<'_, _, f32>("accuracy")
                .unwrap_or(1.0)
        } else {
            let mut total = 0f32;
            let mut div = 0f32;
            for (idx, row) in score_set.iter().enumerate() {
                if let Ok(acc) = row.try_get::<'_, _, f32>("accuracy") {
                    let add = (0.95_f32.powi(idx as i32)) * 100.0;
                    total += acc * add;
                    div += add;
                }
            }
            total / div
        };

        // Calc pp from bp
        self.stats.pp_v2 = {
            let mut total = 0f32;
            for (idx, row) in score_set.iter().enumerate() {
                if let Ok(pp) = row.try_get::<'_, _, f32>("pp_v2") {
                    total += pp * 0.95_f32.powi(idx as i32);
                }
            }
            total
        };
    }

    #[inline(always)]
    pub async fn save_stats(&self, mode: &GameMode, database: &Database) -> bool {
        match database
            .pg
            .execute(
                &format!(
                    r#"UPDATE game_stats.{0} SET 
            total_hits{1} = $1,
            total_score{1} = $2, 
            ranked_score{1} = $3, 
            playcount{1} = $4, 
            playtime{1} = $5, 
            max_combo{1} = $6, 
            accuracy{1} = $7, 
            pp_v2{1} = $8 
            WHERE "id" = $9"#,
                    mode.mode_name(),
                    mode.sub_mod_table()
                ),
                &[
                    &self.stats.total_hits,
                    &self.stats.total_score,
                    &self.stats.ranked_score,
                    &self.stats.playcount,
                    &self.stats.playtime,
                    &self.stats.max_combo,
                    &self.stats.accuracy,
                    &self.stats.pp_v2,
                    &self.id,
                ],
            )
            .await
        {
            Ok(_) => true,
            Err(err) => {
                error!("[Player.Stats] Failed to save to database, err: {:?}", err);
                false
            }
        }
    }

    #[inline(always)]
    pub async fn recalculate_stats(
        &mut self,
        mode: &GameMode,
        database: &Database,
        calculate_pp_acc: bool,
    ) {
        if calculate_pp_acc {
            self.recalculate_pp_acc(&mode.full_name(), database).await;
        };
        self.save_stats(mode, database).await;
        self.stats.calc_rank_from_database(mode, database).await;
        self.cache_stats(&mode);
        self.enqueue(packets::user_stats(&self).await).await;
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
    pub fn get_country_code(&self) -> u8 {
        CountryCodes::from_str(&self.country).unwrap_or(CountryCodes::UN) as u8
    }

    #[inline(always)]
    pub async fn update_mods(
        &mut self,
        game_mode: &GameMode,
        play_mods: &PlayMods,
    ) -> Option<PacketData> {
        if &self.status.game_mode != game_mode || self.status.play_mods.value != play_mods.value {
            self.status.game_mode = game_mode.clone();
            self.status.play_mods = play_mods.clone();
            return Some(packets::user_stats(&self).await);
        }
        None
    }

    #[inline(always)]
    pub async fn check_password_hash(
        &self,
        password_hash: &String,
        argon2_cache: &RwLock<Argon2Cache>,
    ) -> bool {
        // Try read password hash from argon2 cache
        let cached_password_hash = { argon2_cache.read().await.get(&self._password).cloned() };

        // Cache hitted, checking
        if let Some(cached_password_hash) = cached_password_hash {
            debug!("password cache hitted: {}({})", self.name, self.id);
            return &cached_password_hash == password_hash;
        }

        let verify_result = utils::argon2_verify(&self._password, password_hash).await;
        if verify_result {
            // If password is correct, cache it
            // key = argon2 cipher, value = password hash
            argon2_cache
                .write()
                .await
                .insert(self._password.clone(), password_hash.clone());
        }

        verify_result
    }

    #[inline(always)]
    pub fn check_password_argon2(&self, password_argon2: &String) -> bool {
        &self._password == password_argon2
    }

    #[inline(always)]
    pub fn update_password(&mut self, password: &String) {
        self._password = password.clone()
    }

    #[inline(always)]
    pub fn bancho_privileges(privileges: i32) -> i32 {
        let mut bancho_priv = 0;

        if Privileges::Normal.enough(privileges) {
            bancho_priv |= BanchoPrivileges::Player as i32
        }

        if Privileges::Supporter.enough(privileges) {
            bancho_priv |= BanchoPrivileges::Supporter as i32
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

    #[inline(always)]
    pub async fn add_notes(
        &self,
        content: &String,
        note_type: Option<&str>,
        data: Option<&str>,
        added_by: Option<&str>,
        database: &Database,
    ) -> bool {
        match database.pg.execute(
            r#"INSERT INTO "user"."notes" ("user_id","content","data","type","added_by") VALUES ($1, $2, $3, $4, $5)"#, 
            &[&self.id, content, &data, &note_type, &added_by]).await {
            Ok(_) => true,
            Err(err) => {
                error!(
                    "failed to add notes {}({:?}) for player {}({}), err: {:?}; data: {:?}",
                    content, note_type, self.name, self.id, err, data
                );
                false
            }
        }
    }

    #[inline(always)]
    pub async fn hack_detected(
        &mut self,
        hack_id: i32,
        detect_from: &str,
        notification: Option<&str>,
        database: &Database,
    ) {
        const BASE_KEY: &str = "hack_dectected";
        let key = format!("{}_{}_from_{}", BASE_KEY, hack_id, detect_from);

        if self.flag_cache.contains_key(&key) {
            debug!("Hack ({:?}) dected from {} but already added: player {}({}), login_record_id: {}, credit: {}, cheat count: {}.",
            hack_id, detect_from, self.name, self.id, self.login_record_id, self.info.credit, self.info.cheat);
            return;
        }

        self.add_notes(
            &key,
            Some(BASE_KEY),
            Some(&format!(
                "#json#{}",
                json!({
                    "hack_id": hack_id,
                    "detect_from": detect_from,
                    "login_record_id": self.login_record_id,
                    "credit": self.info.credit
                })
            )),
            Some("peace"),
            database,
        )
        .await;

        self.flag_cache.insert(key, Some(hack_id.to_string()));

        // Update info and sync with DB
        self.info
            .set_cheat_with_db(self.info.cheat + 1, database)
            .await;
        self.info
            .set_credit_with_db(self.info.credit - CHEAT_DETECTED_DECREASE_CREDIT, database)
            .await;

        // If have, sent notification to player
        if let Some(notification) = notification {
            self.enqueue(packets::notification(notification)).await;
        }

        warn!(
            "Hack ({:?}) detected warning from {}: player {}({}), login_record_id: {}, credit: {}, cheat: {}.",
            hack_id, detect_from, self.name, self.id, self.login_record_id, self.info.credit, self.info.cheat
        );
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
    pub fn cache_stats(&mut self, mode: &GameMode) {
        let s = self.stats.clone();
        self.stats_cache.insert(mode.clone(), s);
    }

    #[inline(always)]
    pub async fn get_stats_from_database(&self, database: &Database) -> Option<Stats> {
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
            self.status.game_mode.sub_mod_table(),
            self.status.game_mode.mode_name()
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
                    .calc_rank_from_database(&self.status.game_mode, database)
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
    pub async fn once_notification(&mut self, key: &str, notification: &str) {
        if !self.flag_cache.contains_key(key) {
            // send notification to this player once
            self.enqueue(packets::notification(notification)).await;
            self.flag_cache.insert(key.to_owned(), None);
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
