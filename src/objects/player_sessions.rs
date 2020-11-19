#![allow(dead_code)]
use async_std::sync::RwLock;
use uuid::Uuid;

use crate::types::{PlayerHandler, PlayerSessionMap, TokenString};

use super::Player;

pub struct PlayerSessions {
    pub map: PlayerSessionMap,
}

impl PlayerSessions {
    /// Create new PlayerSessions with capacity
    pub fn new(capacity: usize) -> Self {
        PlayerSessions {
            map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
        }
    }

    /// Create token, and login a player into sessions
    pub async fn login(&self, player: Player) -> TokenString {
        let uuid = Uuid::new_v4().to_string();
        let mut player_sessions = self.map.write().await;
        player_sessions.insert(uuid.clone(), player);
        uuid
    }

    /// For debug, get PlayerSessions string
    pub async fn to_string(&self) -> String {
        format!("{:?}", self.map.read().await)
    }

    /// Get a player data (readonly)
    pub async fn get_player_data(&self, token: TokenString) -> Option<Player> {
        let player_sessions = self.map.read().await;
        match player_sessions.get(&token) {
            Some(player) => Some(player.clone()),
            None => None,
        }
    }

    /// Handle a player, then return player data
    pub async fn handle_player(
        &self,
        token: TokenString,
        handler: PlayerHandler,
    ) -> Option<Player> {
        let mut player_sessions = self.map.write().await;
        match player_sessions.get_mut(&token) {
            Some(player) => {
                handler(player);
                Some(player.clone())
            }
            None => None,
        }
    }
}
