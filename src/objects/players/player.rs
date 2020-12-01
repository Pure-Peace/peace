#![allow(dead_code)]
use super::PlayerBase;

use crate::types::PacketData;
use crate::{
    constants::{BanchoPrivileges, Privileges},
    database::Database,
};

use actix_web::web::Data;
use async_std::sync::Mutex;
use chrono::prelude::{DateTime, Local};
use queue::Queue;

#[derive(Debug)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub bancho_privileges: i32,
    pub friends: Vec<i32>,
    pub country: String,
    pub osu_version: String,
    pub utc_offset: i32,
    pub queue: Mutex<Queue<PacketData>>,
    pub login_time: DateTime<Local>,
    pub last_active_time: DateTime<Local>,
}

impl Player {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    pub async fn from_base(base: PlayerBase, osu_version: String, utc_offset: i32) -> Self {
        let now_time = Local::now();
        Player {
            id: base.id,
            name: base.name,
            privileges: base.privileges,
            bancho_privileges: Player::bancho_privileges(base.privileges),
            friends: Vec::new(),
            country: base.country,
            osu_version,
            utc_offset,
            queue: Mutex::new(Queue::new()),
            login_time: now_time,
            last_active_time: now_time,
        }
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
                "error when update_friends_from_database; user: {}({}); err: {:?}",
                self.name, self.id, err
            ),
        };
    }

    /// Enqueue a packet into queue, returns the length of queue
    async fn enqueue(&self, packet_data: PacketData) -> Result<usize, ()> {
        self.queue.lock().await.queue(packet_data)
    }

    async fn dequeue(&self) -> Option<PacketData> {
        self.queue.lock().await.dequeue()
    }

    /// Get the queue data as vec, readonly
    async fn queue_data(&self) -> Vec<PacketData> {
        self.queue.lock().await.vec().clone()
    }

    /// Get the queue size
    async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }

    async fn queue_peek(&self) -> Option<PacketData> {
        self.queue.lock().await.peek()
    }

    async fn queue_is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }
}
