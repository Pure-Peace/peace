use std::{
    fmt,
    sync::{
        atomic::{AtomicI16, Ordering},
        Arc,
    },
};

use async_std::sync::RwLock;
use chrono::{DateTime, Local};
use hashbrown::HashMap;

use crate::{
    objects::{Player, PlayerSessions},
    packets,
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
    pub id_session_map: PlayerIdSessionMap,
    pub player_sessions: Arc<RwLock<PlayerSessions>>,
    pub player_count: AtomicI16,
    pub create_time: DateTime<Local>,
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sessions = vec![];
        if let Some(guard) = self.id_session_map.try_read() {
            for id in guard.keys() {
                sessions.push(*id)
            }
        };
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
    /// Create channel from base object (from database)
    pub async fn from_base(
        base: &ChannelBase,
        player_sessions: Arc<RwLock<PlayerSessions>>,
    ) -> Self {
        Channel {
            name: base.name.to_string(),
            title: base.title.to_string(),
            read_priv: base.read_priv,
            write_priv: base.write_priv,
            auto_join: base.auto_join,
            auto_close: false,
            id_session_map: RwLock::new(HashMap::with_capacity(50)),
            player_sessions,
            player_count: AtomicI16::new(0),
            create_time: Local::now(),
        }
    }

    pub fn new(
        name: String,
        title: String,
        read_priv: i32,
        write_priv: i32,
        auto_join: bool,
        auto_close: bool,
        player_sessions: Arc<RwLock<PlayerSessions>>,
    ) -> Self {
        Channel {
            name,
            title,
            read_priv,
            write_priv,
            auto_join,
            auto_close,
            id_session_map: RwLock::new(HashMap::with_capacity(50)),
            player_sessions,
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
        sender_id: i32,
        msg: &String,
        include_sender: bool,
    ) {
        let packet_data = packets::send_message(sender, sender_id, msg, &self.display_name()).await;
        // For every players in channel
        for (id, player) in self.id_session_map.read().await.iter() {
            // If not include sender, skip sender
            if !include_sender && (*id == sender_id) {
                continue;
            }
            // Send them message
            player.read().await.enqueue(packet_data.clone()).await;
        }
    }

    #[inline(always)]
    pub fn channel_info_packet(&self) -> PacketData {
        packets::channel_info(
            &self.display_name(),
            &self.title,
            self.player_count.load(Ordering::SeqCst),
        )
    }

    #[inline(always)]
    pub async fn update_channel_for_users(&self, player_sessions: Option<&PlayerSessions>) {
        let packet_data = self.channel_info_packet();
        if self.auto_close {
            // Temporary channel: for in channel players
            for player in self.id_session_map.read().await.values() {
                player.read().await.enqueue(packet_data.clone()).await;
            }
        } else if player_sessions.is_some() {
            // Permanent channel: for all players
            player_sessions.unwrap().enqueue_all(&packet_data).await;
        }
    }

    #[inline(always)]
    /// Add a player to this channel
    pub async fn join(&self, player_id: i32, player_sessions: Option<&PlayerSessions>) -> bool {
        let result = if player_sessions.is_none() {
            let player_sessions = self.player_sessions.read().await;
            let id_session_map = player_sessions.id_session_map.read().await;
            match id_session_map.get(&player_id) {
                Some(player) => self.handle_join(player, Some(&*player_sessions)).await,
                None => false,
            }
        } else {
            let id_session_map = player_sessions.unwrap().id_session_map.read().await;
            match id_session_map.get(&player_id) {
                Some(player) => self.handle_join(player, player_sessions).await,
                None => false,
            }
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
    pub async fn handle_join(
        &self,
        player: &Arc<RwLock<Player>>,
        player_sessions: Option<&PlayerSessions>,
    ) -> bool {
        let (player_name, player_id) = {
            let player_cloned = player.clone();
            let mut player = player.write().await;
            if self.id_session_map.read().await.contains_key(&player.id) {
                debug!(
                    "Player {}({}) is already in channel {}!",
                    player.name, player.id, self.name
                );
                return false;
            }

            self.id_session_map
                .write()
                .await
                .insert(player.id, player_cloned);

            player.channels.insert(self.name.to_string());
            self.player_count.fetch_add(1, Ordering::SeqCst);

            // Send it to player's client
            player.enqueue(packets::channel_join(&self.display_name())).await;

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
    pub async fn leave(&self, player_id: i32, player_sessions: Option<&PlayerSessions>) -> bool {
        if !self.id_session_map.read().await.contains_key(&player_id) {
            debug!("Player ({}) is not in channel {}!", player_id, self.name);
            return false;
        }

        let result = self.id_session_map.write().await.remove(&player_id);
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
            player.enqueue(packets::channel_kick(&self.display_name())).await;

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
