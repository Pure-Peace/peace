use std::{
    fmt,
    sync::{
        atomic::{AtomicI16, Ordering},
        Arc,
    },
};

use tokio::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashMap;

use crate::{
    objects::{Player, PlayerSessions},
    types::{PacketData, PlayerIdSessionMap},
};

use super::base::ChannelBase;

pub struct Channel {
    pub name: String,
    pub title: String,
    pub read_priv: i32,
    pub write_priv: i32,
    pub auto_join: bool,
    pub auto_close: bool,
    pub id_map: PlayerIdSessionMap,
    pub player_count: AtomicI16,
    pub create_time: DateTime<Local>,
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sessions = vec![];
        for id in self.id_map.keys() {
            sessions.push(*id)
        }
        f.debug_struct("Channel")
            .field("name", &self.name)
            .field("title", &self.title)
            .field("read_priv", &self.read_priv)
            .field("write_priv", &self.write_priv)
            .field("auto_join", &self.auto_join)
            .field("auto_close", &self.auto_close)
            .field("sessions", &sessions)
            .field("player_count", &self.player_count)
            .field("create_time", &self.create_time)
            .finish()
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        debug!("Channel {} object dropped!", self.name);
    }
}

impl PartialEq for Channel {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Channel {
    #[inline(always)]
    /// Create channel from base object (from database);
    ///
    /// permanent channels
    ///
    pub async fn from_base(base: &ChannelBase) -> Self {
        Channel {
            name: base.name.to_string(),
            title: base.title.to_string(),
            read_priv: base.read_priv,
            write_priv: base.write_priv,
            auto_join: base.auto_join,
            auto_close: false,
            id_map: HashMap::with_capacity(50),
            player_count: AtomicI16::new(0),
            create_time: Local::now(),
        }
    }

    #[inline(always)]
    pub fn new(
        name: String,
        title: String,
        read_priv: i32,
        write_priv: i32,
        auto_join: bool,
        auto_close: bool,
    ) -> Self {
        Channel {
            name,
            title,
            read_priv,
            write_priv,
            auto_join,
            auto_close,
            id_map: HashMap::with_capacity(50),
            player_count: AtomicI16::new(0),
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
        sender: &String,
        sender_u: &Option<String>,
        sender_id: i32,
        msg: &String,
        include_sender: bool,
    ) {
        let sender_u = sender_u.as_ref().unwrap_or(sender);
        // For every players in channel
        for (id, player) in self.id_map.iter() {
            // If not include sender, skip sender
            if !include_sender && (*id == sender_id) {
                continue;
            }
            // Send them message
            let p = player.read().await;
            p.enqueue(bancho_packets::send_message(
                if p.settings.display_u_name {
                    &sender_u
                } else {
                    sender
                },
                sender_id,
                msg,
                &self.display_name(),
            ))
            .await;
        }
    }

    #[inline(always)]
    pub fn channel_info_packet(&self) -> PacketData {
        bancho_packets::channel_info(
            &self.display_name(),
            &self.title,
            self.player_count.load(Ordering::SeqCst),
        )
    }

    #[inline(always)]
    pub async fn update_channel_for_users(&self, player_sessions: Option<&PlayerSessions>) {
        let packet_data = self.channel_info_packet();
        if player_sessions.is_none() {
            // Temporary channel: for in channel players
            for player in self.id_map.values() {
                player.read().await.enqueue(packet_data.clone()).await;
            }
        } else {
            // Permanent channel: for all players
            player_sessions.unwrap().enqueue_all(&packet_data).await;
        }
    }

    #[inline(always)]
    /// Add a player to this channel by player_id
    pub async fn join_by_player_id(
        &mut self,
        player_id: i32,
        player_sessions: &PlayerSessions,
        broadcast_channel_update: bool,
    ) -> bool {
        let p = player_sessions.get_player_by_id(player_id).await;
        let result = match p {
            Some(player) => {
                self.join_player(
                    player,
                    if broadcast_channel_update {
                        Some(player_sessions)
                    } else {
                        None
                    },
                )
                .await
            }
            None => false,
        };

        if !result {
            debug!(
                "Failed to join Channel ({}), Player ({}) not exists!",
                self.name, player_id
            );
        };

        result
    }

    #[inline(always)]
    /// Join a player into channel
    ///
    /// If it's a permanent channel, should pass Some(player_sessions),
    /// it means channel info will broadcast to all players.
    /// otherwise, just pass None, channel info will only send to players in channel.
    pub async fn join_player(
        &mut self,
        player: Arc<RwLock<Player>>,
        player_sessions: Option<&PlayerSessions>,
    ) -> bool {
        let (player_name, player_id) = {
            let player_cloned = player.clone();
            let mut player = player.write().await;
            if self.id_map.contains_key(&player.id) {
                debug!(
                    "Player {}({}) is already in channel {}!",
                    player.name, player.id, self.name
                );
                return false;
            }

            self.id_map.insert(player.id, player_cloned);

            player.channels.insert(self.name.to_string());
            self.player_count.fetch_add(1, Ordering::SeqCst);

            // Send it to player's client
            player
                .enqueue(bancho_packets::channel_join(&self.display_name()))
                .await;

            (player.name.clone(), player.id)
        };

        // Update channel info for users
        self.update_channel_for_users(player_sessions).await;

        debug!(
            "Player {}({}) has joined channel {}!",
            player_name, player_id, self.name
        );
        return true;
    }

    #[inline(always)]
    /// Remove a player from this channel
    ///
    /// If it's a permanent channel, should pass Some(player_sessions),
    /// it means channel info will broadcast to all players.
    /// otherwise, just pass None, channel info will only send to players in channel.
    pub async fn leave(
        &mut self,
        player_id: i32,
        player_sessions: Option<&PlayerSessions>,
    ) -> bool {
        if !self.id_map.contains_key(&player_id) {
            debug!("Player ({}) is not in channel {}!", player_id, self.name);
            return false;
        }

        let result = self.id_map.remove(&player_id);
        if result.is_none() {
            warn!(
                "Failed to remove Player ({}) from Channel ({})!",
                player_id, self.name
            );
            return false;
        };

        let (player_name, player_id) = {
            let player = result.unwrap();
            self.player_count.fetch_sub(1, Ordering::SeqCst);
            let mut player = player.write().await;

            player.channels.remove(&self.name);
            // Send it to player's client
            player
                .enqueue(bancho_packets::channel_kick(&self.display_name()))
                .await;

            (player.name.clone(), player.id)
        };

        // Update channel info for users
        self.update_channel_for_users(player_sessions).await;

        debug!(
            "Player {}({}) has been removed from channel {}!",
            player_name, player_id, self.name
        );
        return true;
    }
}
