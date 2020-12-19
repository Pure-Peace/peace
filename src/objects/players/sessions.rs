#![allow(dead_code)]
use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::Local;
use std::future::Future;
use uuid::Uuid;

use crate::{
    database::Database,
    packets,
    types::{ChannelList, PacketData, PlayerIdSessionMap, PlayerSessionMap, TokenString, UserId},
};

use super::{Player, PlayerData};

pub struct PlayerSessions {
    pub map: PlayerSessionMap,
    pub id_session_map: PlayerIdSessionMap,
    pub player_count: u32,
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
            player_count: 0,
            database,
        }
    }

    #[inline(always)]
    /// Create new token, and login a player into PlayerSessions
    pub async fn login(&mut self, player: Player) -> TokenString {
        let token = Uuid::new_v4().to_string();
        self.handle_login(player, token).await
    }

    #[inline(always)]
    /// Login a player into PlayerSessions with a token
    pub async fn handle_login(&mut self, player: Player, token: TokenString) -> TokenString {
        let player_id = player.id;
        // Get locks
        let (mut map, mut id_session_map) =
            (self.map.write().await, self.id_session_map.write().await);
        // Insert into
        map.insert(token.clone(), player);
        id_session_map.insert(player_id, token.clone());
        self.player_count += 1;
        token
    }

    /// Logout a player from the PlayerSessions
    pub async fn logout(
        &mut self,
        token: &TokenString,
        channel_list: Option<&Data<RwLock<ChannelList>>>,
    ) -> Option<(TokenString, Player)> {
        let logout_start = std::time::Instant::now();
        // Get locks
        let (mut map, mut id_session_map) =
            (self.map.write().await, self.id_session_map.write().await);
        // Logout
        match map.remove_entry(token) {
            Some((token_string, mut player)) => {
                // Remove and drop locks
                id_session_map.remove(&player.id);
                drop(map);
                drop(id_session_map);
                self.player_count -= 1;

                // Enqueue logout packet to all players
                self.enqueue_all(&packets::user_logout(player.id)).await;
                match channel_list {
                    Some(channel_list) => {
                        for channel in channel_list.write().await.values_mut() {
                            if player.channels.contains(&channel.name) {
                                channel.leave(&mut player).await;
                            }
                        }
                    }
                    _ => {}
                }
                player.update_logout_time(&self.database).await;

                let logout_end = logout_start.elapsed();
                info!(
                    "user {}({}) has logouted; time spent: {:.2?}",
                    player.name, player.id, logout_end
                );
                Some((token_string, player))
            }
            None => None,
        }
    }

    #[inline(always)]
    pub async fn enqueue_all(&self, packet_data: &PacketData) {
        for player in self.map.read().await.values() {
            player.enqueue(packet_data.clone()).await;
        }
    }

    #[inline(always)]
    pub async fn enqueue_by_token(&self, token: &TokenString, packet_data: PacketData) -> bool {
        if let Some(player) = self.map.read().await.get(token) {
            player.enqueue(packet_data).await;
            return true;
        }
        false
    }

    #[inline(always)]
    pub async fn enqueue_by_id(&self, user_id: &UserId, packet_data: PacketData) -> bool {
        let token = match self.id_session_map.read().await.get(user_id) {
            Some(token) => token.clone(),
            None => return false,
        };

        if let Some(player) = self.map.read().await.get(&token) {
            player.enqueue(packet_data).await;
            return true;
        }
        false
    }

    #[inline(always)]
    pub async fn get_token_by_id(&self, user_id: &UserId) -> Option<String> {
        match self.id_session_map.read().await.get(user_id) {
            Some(token) => Some(token.clone()),
            None => None,
        }
    }

    #[inline(always)]
    pub async fn get_id_by_token(&self, token: &TokenString) -> Option<i32> {
        match self.map.read().await.get(token) {
            Some(player) => Some(player.id.clone()),
            None => None,
        }
    }

    #[inline(always)]
    pub async fn get_id_by_name(&self, name: &str) -> Option<i32> {
        match self
            .map
            .read()
            .await
            .values()
            .find(|player| player.name == name)
        {
            Some(player) => Some(player.id.clone()),
            None => None,
        }
    }

    #[inline(always)]
    /// Token is exists or not
    pub async fn token_is_exists(&self, token: &TokenString) -> bool {
        self.map.read().await.contains_key(token)
    }

    #[inline(always)]
    pub async fn id_is_exists(&self, id: &UserId) -> bool {
        self.id_session_map.read().await.contains_key(&id)
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
    pub async fn logout_with_id(
        &mut self,
        user_id: UserId,
        channel_list: Option<&Data<RwLock<ChannelList>>>,
    ) -> Option<(TokenString, Player)> {
        let token = match self.id_session_map.read().await.get(&user_id) {
            Some(token) => token.to_string(),
            None => return None,
        };
        self.logout(&token, channel_list).await
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
    /// Get a player data (readonly)
    pub async fn get_player_data(&self, token: &TokenString) -> Option<PlayerData> {
        match self.map.read().await.get(token) {
            Some(player) => Some(PlayerData::from(player)),
            None => None,
        }
    }

    #[inline(always)]
    /// Handle a player, then return player data
    pub async fn handle_player_get<F>(
        &self,
        token: &TokenString,
        handler: F,
    ) -> Result<PlayerData, ()>
    where
        F: FnOnce(&mut Player) -> Option<()>,
    {
        match self.map.write().await.get_mut(token) {
            Some(player) => match handler(player) {
                Some(()) => Ok(PlayerData::from(player)),
                None => Err(()),
            },
            None => Err(()),
        }
    }

    #[inline(always)]
    /// Handle a player
    pub async fn handle_player<F>(&self, token: &TokenString, handler: F) -> Result<(), ()>
    where
        F: FnOnce(&mut Player) -> Option<()>,
    {
        match self.map.write().await.get_mut(token) {
            Some(player) => match handler(player) {
                Some(()) => Ok(()),
                None => Err(()),
            },
            None => Err(()),
        }
    }
}
