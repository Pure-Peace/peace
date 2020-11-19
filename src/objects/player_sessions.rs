#![allow(dead_code)]
use async_std::sync::RwLock;
use uuid::Uuid;

use crate::types::{PlayerHandler, PlayerIdSessionMap, PlayerSessionMap, TokenString, UserId};

use super::Player;

pub struct PlayerSessions {
    pub map: PlayerSessionMap,
    pub id_session_map: PlayerIdSessionMap,
}

impl PlayerSessions {
    /// Create new PlayerSessions with capacity
    pub fn new(capacity: usize) -> Self {
        PlayerSessions {
            map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
            id_session_map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
        }
    }

    /// Create new token, and login a player into PlayerSessions
    pub async fn login(&self, player: Player) -> TokenString {
        let token = Uuid::new_v4().to_string();
        let player_id = player.id;
        self.map.write().await.insert(token.clone(), player);
        self.id_session_map
            .write()
            .await
            .insert(player_id, token.clone());
        token
    }

    /// Login a player into PlayerSessions with a token
    pub async fn login_with_token(&self, player: Player, token: TokenString) -> TokenString {
        let player_id = player.id;
        self.map.write().await.insert(token.clone(), player);
        self.id_session_map
            .write()
            .await
            .insert(player_id, token.clone());
        token
    }

    /// Logout a player from the PlayerSessions
    pub async fn logout(&self, token: TokenString) -> Option<(TokenString, Player)> {
        match self.map.write().await.remove_entry(&token) {
            Some((token_string, player)) => {
                self.id_session_map.write().await.remove(&player.id);
                Some((token_string, player))
            }
            None => None,
        }
    }

    /// Logout a player from the PlayerSessions with user id
    pub async fn logout_with_id(&self, user_id: UserId) -> Option<(TokenString, Player)> {
        let token = match self.id_session_map.read().await.get(&user_id) {
            Some(token) => token.to_string(),
            None => return None
        };
        self.logout(token).await
    }

    /// Use user_id check user is exists
    pub async fn user_is_logined(&self, user_id: UserId) -> bool {
        self.id_session_map.read().await.contains_key(&user_id)
    }

    /// For debug, get PlayerSessions.map to string
    pub async fn map_to_string(&self) -> String {
        format!("{:?}", self.map.read().await)
    }

    /// For debug, get PlayerSessions.id_session_map to string
    pub async fn id_map_to_string(&self) -> String {
        format!("{:?}", self.id_session_map.read().await)
    }

    /// Token is exists or not
    pub async fn token_is_exists(&self, token: TokenString) -> bool {
        self.map.read().await.contains_key(&token)
    }

    /// Get a player data (readonly)
    pub async fn get_player_data(&self, token: TokenString) -> Option<Player> {
        match self.map.read().await.get(&token) {
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
        match self.map.write().await.get_mut(&token) {
            Some(player) => {
                handler(player);
                Some(player.clone())
            }
            None => None,
        }
    }
}
