#![allow(dead_code)]
use std::sync::{Arc, Weak};

use super::PlayerBase;

use crate::{
    constants::{Action, ClientInfo, GameMode, PlayMods, PresenceFilter},
    objects::Channel,
    types::{Location, PacketData},
};
use crate::{
    constants::{BanchoPrivileges, Privileges},
    database::Database,
};

use actix_web::web::Data;
use async_std::sync::{Mutex, RwLock};
use chrono::prelude::{DateTime, Local};
use hashbrown::{HashMap, HashSet};
use queue::Queue;

#[derive(Debug, Clone)]
pub struct Stats {
    pub rank: i32,
    pub performance_v1: i16,
    pub performance_v2: i16,
    pub accuracy: f32,
    pub total_score: i64,
    pub ranked_score: i64,
    pub playcount: i32,
    pub playtime: i64,
    pub max_combo: i32,
    pub update_time: DateTime<Local>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            rank: 100000,
            performance_v1: 0,
            performance_v2: 0,
            accuracy: 0.0,
            total_score: 0,
            ranked_score: 0,
            playcount: 0,
            playtime: 0,
            max_combo: 0,
            update_time: Local::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Status {
    pub action: Action,
    pub info: String,
    pub playing_beatmap_id: i32,
    pub playing_beatmap_md5: String,
    pub play_mods: PlayMods,
    pub game_mode: GameMode,
    pub update_time: DateTime<Local>,
}

impl Status {
    pub fn new() -> Self {
        Status {
            action: Action::Idle,
            info: String::new(),
            playing_beatmap_id: 0,
            playing_beatmap_md5: String::new(),
            play_mods: PlayMods::NoMod,
            game_mode: GameMode::Std,
            update_time: Local::now(),
        }
    }
}

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
    pub location: Location,
    pub stats: Stats,
    pub status: Status,
    pub queue: Mutex<Queue<PacketData>>,
    pub channels: HashSet<String>,
    pub login_time: DateTime<Local>,
    pub login_record_id: i64,
    pub last_active_time: DateTime<Local>,
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
            ip,
            address_id,
            address_similarity,
            only_friend_pm_allowed: client_info.only_friend_pm_allowed,
            presence_filter: PresenceFilter::None,
            display_city: client_info.display_city,
            osu_version: client_info.osu_version,
            utc_offset: client_info.utc_offset as u8,
            location: (0.0, 0.0),
            stats: Stats::new(),
            status: Status::new(),
            queue: Mutex::new(Queue::new()),
            channels: HashSet::new(),
            login_time: now_time,
            login_record_id: -1,
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
        play_mods: PlayMods,
        game_mode: GameMode,
    ) -> Option<()> {
        self.status.action = action;
        self.status.info = info;
        self.status.playing_beatmap_md5 = playing_beatmap_md5;
        self.status.playing_beatmap_id = playing_beatmap_id;
        self.status.play_mods = play_mods;
        self.status.game_mode = game_mode;
        self.status.update_time = Local::now();
        Some(())
    }

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

    pub async fn update_friends_from_database(&mut self, database: &Data<Database>) {
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
                "Error when update_friends_from_database; user: {}({}); err: {:?}",
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
    pub async fn dequeue(&self) -> Option<PacketData> {
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
