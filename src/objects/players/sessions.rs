#![allow(dead_code)]
use async_std::sync::RwLock;
use chrono::Local;
use uuid::Uuid;

use crate::{
    database::Database,
    types::{PlayerHandler, PlayerIdSessionMap, PlayerSessionMap, TokenString, UserId},
};

use super::{Player, PlayerData};

pub struct PlayerSessions {
    pub map: PlayerSessionMap,
    pub id_session_map: PlayerIdSessionMap,
    database: Database,
}

impl PlayerSessions {
    /// Create new PlayerSessions with a default capacity
    /// Automatically expand when capacity is exceeded
    pub fn new(capacity: usize, database: Database) -> Self {
        PlayerSessions {
            /// Key: token, Value: Player
            map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
            /// Key: Player.id, Value: token
            id_session_map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
            database,
        }
    }

    #[inline(always)]
    /// Create new token, and login a player into PlayerSessions
    pub async fn login(&self, player: Player) -> TokenString {
        let token = Uuid::new_v4().to_string();
        self.login_with_token(player, token).await
    }

    #[inline(always)]
    /// Login a player into PlayerSessions with a token
    pub async fn login_with_token(&self, player: Player, token: TokenString) -> TokenString {
        let player_id = player.id;
        // Get locks
        let (mut map, mut id_session_map) =
            (self.map.write().await, self.id_session_map.write().await);
        // Insert into
        map.insert(token.clone(), player);
        id_session_map.insert(player_id, token.clone());
        token
    }

    /// Logout a player from the PlayerSessions
    pub async fn logout(&self, token: &TokenString) -> Option<(TokenString, Player)> {
        // Get locks
        let (mut map, mut id_session_map) =
            (self.map.write().await, self.id_session_map.write().await);
        // Logout
        match map.remove_entry(token) {
            Some((token_string, player)) => {
                drop(map);
                id_session_map.remove(&player.id);
                drop(id_session_map);
                // If user has login record id, record logout time
                if player.login_record_id > 0 {
                    self.database
                        .pg
                        .execute(
                            r#"UPDATE "user_records"."login" 
                                    SET "logout_time" = now() 
                                    WHERE "id" = $1;"#,
                            &[&player.login_record_id],
                        )
                        .await
                        .unwrap_or_else(|err| {
                            error!(
                                "failed to update user {}({})'s logout_time, error: {:?}",
                                player.name, player.id, err
                            );
                            0
                        });
                }
                Some((token_string, player))
            }
            None => None,
        }
    }

    #[inline(always)]
    /// Logout a player from the PlayerSessions with user id
    ///
    /// Think, why not use the following code?
    /// Because, passing a reference to the token directly will result in the read lock not being released, thus triggering a deadlock.
    /// ```
    /// match self.id_session_map.read().await.get(&user_id) {
    ///     Some(token) => self.logout(token).await,
    ///     None => None,
    /// }
    /// ```
    pub async fn logout_with_id(&self, user_id: UserId) -> Option<(TokenString, Player)> {
        let token = match self.id_session_map.read().await.get(&user_id) {
            Some(token) => token.to_string(),
            None => return None,
        };
        self.logout(&token).await
    }

    #[inline(always)]
    pub async fn deactive_token_list(&self, session_timeout: i64) -> Vec<TokenString> {
        let now_timestamp = Local::now().timestamp();
        self.map
            .read()
            .await
            .iter()
            .filter(|(_, player)| {
                now_timestamp - player.last_active_time.timestamp() > session_timeout
            })
            .map(|(token, _)| token.to_string())
            .collect()
    }

    #[inline(always)]
    /// Use user_id check user is exists
    pub async fn user_is_logined(&self, user_id: UserId) -> bool {
        self.id_session_map.read().await.contains_key(&user_id)
    }

    #[inline(always)]
    /// For debug, get PlayerSessions.map to string
    pub async fn map_to_string(&self) -> String {
        format!("{:?}", self.map.read().await)
    }

    #[inline(always)]
    /// For debug, get PlayerSessions.id_session_map to string
    pub async fn id_map_to_string(&self) -> String {
        format!("{:?}", self.id_session_map.read().await)
    }

    #[inline(always)]
    /// Token is exists or not
    pub async fn token_is_exists(&self, token: TokenString) -> bool {
        self.map.read().await.contains_key(&token)
    }

    #[inline(always)]
    /// Get a player data (readonly)
    pub async fn get_player_data(&self, token: TokenString) -> Option<PlayerData> {
        match self.map.read().await.get(&token) {
            Some(player) => Some(PlayerData::from(player)),
            None => None,
        }
    }

    #[inline(always)]
    /// Handle a player, then return player data
    pub async fn handle_player(
        &self,
        token: TokenString,
        handler: PlayerHandler,
    ) -> Option<PlayerData> {
        match self.map.write().await.get_mut(&token) {
            Some(player) => {
                handler(player);
                Some(PlayerData::from(player))
            }
            None => None,
        }
    }
}
