use std::{fmt, sync::Weak};

use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashSet;

use crate::{objects::PlayerSessions, packets};

use crate::objects::Player;

use super::base::ChannelBase;

pub struct Channel {
    pub name: String,
    pub title: String,
    pub read_priv: i32,
    pub write_priv: i32,
    pub auto_join: bool,
    pub auto_close: bool,
    pub players: RwLock<HashSet<i32>>,
    pub player_count: i16,
    pub join_count: u32,
    pub leave_count: u32,
    pub create_time: DateTime<Local>,
    player_sessions: Data<RwLock<PlayerSessions>>,
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nChannel: {{ name: {}, title: {}, read_priv: {}, write_priv: {}, auto_join: {}, auto_close: {}, players: {:?}, player_count: {}, join_count: {}, leave_count: {}, create_time: {:?} }}", 
        self.name, 
        self.title, 
        self.read_priv, 
        self.write_priv, 
        self.auto_join, 
        self.auto_close, 
        self.players, 
        self.player_count,
        self.join_count,
        self.leave_count,
        self.create_time)
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Channel {
    /// Create channel from base object (from database)
    pub async fn from_base(
        base: &ChannelBase,
        player_sessions: Data<RwLock<PlayerSessions>>,
    ) -> Self {
        Channel {
            name: base.name.to_string(),
            title: base.title.to_string(),
            read_priv: base.read_priv,
            write_priv: base.write_priv,
            auto_join: base.auto_join,
            auto_close: false,
            players: RwLock::new(HashSet::new()),
            player_count: 0,
            join_count: 0,
            leave_count: 0,
            player_sessions,
            create_time: Local::now()
        }
    }

    /// Channel display name
    pub fn display_name(&self) -> String {
        match &self.name {
            n if n.starts_with("#spec_") => "#spectator".to_string(),
            n if n.starts_with("#multi_") => "#multiplayer".to_string(),
            n => n.to_string(),
        }
    }

    /// Send message to every player in channel
    pub async fn broadcast(&self, sender: &String, sender_id: i32, msg: &String, include_sender: bool) {
        let player_sessions = self.player_sessions.read().await;
        let packet_data = packets::send_message(sender, sender_id, msg, &self.name).await;
        // For every players in channel
        let players = self.players.read().await;
        for player in player_sessions
            .map
            .read()
            .await
            .values()
            .filter(|player| {
                if !include_sender && (player.id == sender_id) {
                    return false;
                }
                players.contains(&player.id)
            })
        {
            // Send them message
            player.enqueue(packet_data.clone()).await;
        }
    }

    /// Add a player to this channel
    pub async fn join(&mut self, player: &mut Player) -> bool {
        if self.players.write().await.insert(player.id) {
            player.channels.insert(self.name.to_string());
            self.player_count += 1;
            self.join_count += 1;
            debug!(
                "Player {}({}) has joined channel {}!",
                player.name, player.id, self.name
            );
            return true;
        }

        debug!(
            "Player {}({}) is already in channel {}!",
            player.name, player.id, self.name
        );
        return false;
    }

    /// Remove a player from this channel
    pub async fn leave(&mut self, player: &mut Player) -> bool {
        if self.players.write().await.remove(&player.id) {
            player.channels.remove(&self.name);
            self.player_count -= 1;
            self.leave_count += 1;
            debug!(
                "Player {}({}) has been removed from channel {}!",
                player.name, player.id, self.name
            );
            return true;
        }

        debug!(
            "Player {}({}) is not in channel {}!",
            player.name, player.id, self.name
        );
        return false;
    }
}
