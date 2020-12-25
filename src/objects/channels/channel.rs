use std::{fmt, sync::Weak};

use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashSet;

use crate::{objects::PlayerSessions, packets, types::PacketData};

use crate::objects::Player;

use super::base::ChannelBase;

#[derive(Debug)]
pub struct Channel {
    pub name: String,
    pub title: String,
    pub read_priv: i32,
    pub write_priv: i32,
    pub auto_join: bool,
    pub auto_close: bool,
    pub players: RwLock<HashSet<i32>>,
    pub player_count: i16,
    pub create_time: DateTime<Local>,
}

impl PartialEq for Channel {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Channel {
    #[inline(always)]
    /// Create channel from base object (from database)
    pub async fn from_base(base: &ChannelBase) -> Self {
        Channel {
            name: base.name.to_string(),
            title: base.title.to_string(),
            read_priv: base.read_priv,
            write_priv: base.write_priv,
            auto_join: base.auto_join,
            auto_close: false,
            players: RwLock::new(HashSet::new()),
            player_count: 0,
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    /// Channel display name
    pub fn display_name(&self) -> String {
        match &self.name {
            n if n.starts_with("#spec_") => "#spectator".to_string(),
            n if n.starts_with("#multi_") => "#multiplayer".to_string(),
            n => n.to_string(),
        }
    }

    #[inline(always)]
    /// Send message to every player in channel
    pub async fn broadcast(
        &self,
        player_sessions: &Data<RwLock<PlayerSessions>>,
        sender: &String,
        sender_id: i32,
        msg: &String,
        include_sender: bool,
    ) {
        let player_sessions = player_sessions.read().await;
        let packet_data = packets::send_message(sender, sender_id, msg, &self.name).await;
        // For every players in channel
        let players = self.players.read().await;
        for player in player_sessions.map.read().await.values().filter(|player| {
            if !include_sender && (player.id == sender_id) {
                return false;
            }
            players.contains(&player.id)
        }) {
            // Send them message
            player.enqueue(packet_data.clone()).await;
        }
    }

    #[inline(always)]
    pub async fn channel_info_packet(&self) -> PacketData {
        packets::channel_info(&self.name, &self.title, self.player_count)
    }

    #[inline(always)]
    pub async fn update_channel_for_users(&self, player_sessions: &PlayerSessions) {
        let packet_data = self.channel_info_packet().await;
        match self.auto_close {
            // Temporary channel: for in channel players
            true => {
                for user_id in self.players.read().await.iter() {
                    player_sessions
                        .enqueue_by_id(user_id, packet_data.clone())
                        .await;
                }
            }
            // Permanent channel: for all players
            false => {
                player_sessions.enqueue_all(&packet_data).await;
            }
        }
    }

    #[inline(always)]
    /// Add a player to this channel
    pub async fn join(&mut self, player: &mut Player) -> bool {
        if self.players.write().await.insert(player.id) {
            player.channels.insert(self.name.to_string());
            self.player_count += 1;

            // Send it to player's client
            player.enqueue(packets::channel_join(&self.name)).await;

            // Update channel info for users
            // self.update_channel_for_users(&player_sessions).await;

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

    #[inline(always)]
    /// Remove a player from this channel
    pub async fn leave(&mut self, player: &mut Player) -> bool {
        if self.players.write().await.remove(&player.id) {
            player.channels.remove(&self.name);
            self.player_count -= 1;

            // Send it to player's client
            player.enqueue(packets::channel_kick(&self.name)).await;

            // Update channel info for users
            // self.update_channel_for_users(&player_sessions).await;

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
