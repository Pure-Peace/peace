use actix_web::web::Data;
use async_std::sync::RwLock;
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
    player_sessions: Data<RwLock<PlayerSessions>>,
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
            player_sessions,
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
    pub async fn broadcast(&self, sender: &Player, msg: String) {
        let player_sessions = self.player_sessions.read().await;
        let packet_data = packets::send_message(&sender.name, sender.id, &msg, &self.name);
        // For every players in channel
        let players = self.players.read().await;
        for player in player_sessions
            .map
            .read()
            .await
            .values()
            .filter(|player| players.contains(&player.id))
        {
            // Send them message
            player.enqueue(packet_data.clone()).await;
        }
    }

    /// Add a player to this channel
    pub async fn join(&mut self, player: &Player) -> bool {
        let mut players = self.players.write().await;
        if !players.contains(&player.id) {
            players.insert(player.id);
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
    pub async fn leave(&mut self, player: &Player) -> bool {
        let mut players = self.players.write().await;
        if players.remove(&player.id) {
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
