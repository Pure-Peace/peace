#![allow(dead_code)]
use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::Local;
use std::{
    future::Future,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use uuid::Uuid;

use crate::{
    database::Database,
    packets,
    types::{
        ChannelList, PacketData, PlayerIdSessionMap, PlayerNameSessionMap, PlayerSessionMap,
        TokenString, UserId,
    },
};

use super::{Player, PlayerData};

pub struct PlayerSessions {
    pub token_map: PlayerSessionMap,
    pub id_session_map: PlayerIdSessionMap,
    pub name_session_map: PlayerNameSessionMap,
    pub player_count: AtomicU32,
    database: Database,
}

impl PlayerSessions {
    /// Create new PlayerSessions with a default capacity
    /// Automatically expand when capacity is exceeded
    pub fn new(capacity: usize, database: Database) -> Self {
        PlayerSessions {
            /// Key: token, Value: Arc<RwLock<Player>>
            token_map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
            /// Key: Player.id, Value: Arc<RwLock<Player>>
            id_session_map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
            /// Key: Player.name Value: Arc<RwLock<Player>>
            name_session_map: RwLock::new(hashbrown::HashMap::with_capacity(capacity)),
            player_count: AtomicU32::new(0),
            database,
        }
    }

    #[inline(always)]
    /// Login a player into PlayerSessions
    pub async fn login(&self, player: Player) -> TokenString {
        let token = player.token.clone();
        let player_name = player.name.clone();
        let player_id = player.id;

        {
            let arc_player = Arc::new(RwLock::new(player));

            // Get locks
            let (mut token_map, mut id_session_map, mut name_session_map) = (
                self.token_map.write().await,
                self.id_session_map.write().await,
                self.name_session_map.write().await,
            );
            // Insert into
            token_map.insert(token.clone(), arc_player.clone());
            id_session_map.insert(player_id, arc_player.clone());
            name_session_map.insert(player_name, arc_player.clone());
        }
        self.player_count.fetch_add(1, Ordering::SeqCst);
        token
    }

    /// Logout a player from the PlayerSessions
    pub async fn logout(
        &self,
        token: &TokenString,
        channel_list: Option<&Data<RwLock<ChannelList>>>,
    ) -> Option<Player> {
        let logout_start = std::time::Instant::now();
        // Get locks
        let (mut token_map, mut id_session_map, mut name_session_map) = (
            self.token_map.write().await,
            self.id_session_map.write().await,
            self.name_session_map.write().await,
        );
        // Try Logout
        match token_map.remove(token) {
            Some(arc_player) => {
                // Remove and drop locks
                {
                    let (player_id, player_name) = {
                        let player = arc_player.read().await;
                        (player.id, player.name.clone())
                    };
                    id_session_map.remove(&player_id);
                    name_session_map.remove(&player_name);
                    drop(token_map);
                    drop(id_session_map);
                    drop(name_session_map);
                }

                let mut player = Arc::try_unwrap(arc_player).unwrap().into_inner();
                self.player_count.fetch_sub(1, Ordering::SeqCst);

                // Enqueue logout packet to all players
                self.enqueue_all(&packets::user_logout(player.id)).await;
                match channel_list {
                    Some(channel_list) => {
                        for channel in channel_list.write().await.values_mut() {
                            if player.channels.contains(&channel.name) {
                                channel.leave(&mut player).await;
                                channel.update_channel_for_users(&self).await;
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

                Some(player)
            }
            None => None,
        }
    }

    #[inline(always)]
    pub async fn enqueue_all(&self, packet_data: &PacketData) {
        for player in self.token_map.read().await.values() {
            player.write().await.enqueue(packet_data.clone()).await;
        }
    }

    #[inline(always)]
    pub async fn enqueue_by_token(&self, token: &TokenString, packet_data: PacketData) -> bool {
        if let Some(player) = self.token_map.read().await.get(token) {
            player.write().await.enqueue(packet_data).await;
            return true;
        }
        false
    }

    #[inline(always)]
    pub async fn enqueue_by_id(&self, user_id: &UserId, packet_data: PacketData) -> bool {
        if let Some(player) = self.id_session_map.read().await.get(user_id) {
            player.write().await.enqueue(packet_data).await;
            return true;
        }
        false
    }

    #[inline(always)]
    pub async fn get_token_by_id(&self, user_id: &UserId) -> Option<String> {
        match self.id_session_map.read().await.get(user_id) {
            Some(player) => Some(player.read().await.token.clone()),
            None => None,
        }
    }

    #[inline(always)]
    pub async fn get_id_by_token(&self, token: &TokenString) -> Option<i32> {
        match self.token_map.read().await.get(token) {
            Some(player) => Some(player.read().await.id.clone()),
            None => None,
        }
    }

    #[inline(always)]
    pub async fn get_id_by_name(&self, name: &str) -> Option<i32> {
        match self.name_session_map.read().await.get(name) {
            Some(player) => Some(player.read().await.id.clone()),
            None => None,
        }
    }

    #[inline(always)]
    /// Token is exists or not
    pub async fn token_is_exists(&self, token: &TokenString) -> bool {
        self.token_map.read().await.contains_key(token)
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
        &self,
        user_id: UserId,
        channel_list: Option<&Data<RwLock<ChannelList>>>,
    ) -> Option<Player> {
        let token = match self.id_session_map.read().await.get(&user_id) {
            Some(player) => player.read().await.token.to_string(),
            None => return None,
        };
        self.logout(&token, channel_list).await
    }

    #[inline(always)]
    pub async fn deactive_token_list(&self, session_timeout: i64) -> Vec<TokenString> {
        let now_timestamp = Local::now().timestamp();

        let mut vec = vec![];
        for (token, player) in self.token_map.read().await.iter() {
            if now_timestamp - player.read().await.last_active_time.timestamp() > session_timeout {
                vec.push(token.clone())
            }
        }
        vec
    }

    #[inline(always)]
    /// Use user_id check user is exists
    pub async fn user_is_logined(&self, user_id: UserId) -> bool {
        self.id_session_map.read().await.contains_key(&user_id)
    }

    #[inline(always)]
    /// For debug, get PlayerSessions.map to string
    pub async fn map_to_string(&self) -> String {
        format!("{:?}", self.token_map.read().await)
    }

    #[inline(always)]
    /// For debug, get PlayerSessions.id_session_map to string
    pub async fn id_map_to_string(&self) -> String {
        format!("{:?}", self.id_session_map.read().await)
    }

    #[inline(always)]
    /// Get a player data (readonly)
    pub async fn get_player_data(&self, token: &TokenString) -> Option<PlayerData> {
        match self.token_map.read().await.get(token) {
            Some(player) => Some(PlayerData::from(&*player.read().await)),
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
        match self.token_map.write().await.get_mut(token) {
            Some(player) => {
                let mut player = player.write().await;
                match handler(&mut *player) {
                    Some(()) => Ok(PlayerData::from(&mut *player)),
                    None => Err(()),
                }
            }
            None => Err(()),
        }
    }

    #[inline(always)]
    /// Handle a player
    pub async fn handle_player<F>(&self, token: &TokenString, handler: F) -> Result<(), ()>
    where
        F: FnOnce(&mut Player) -> Option<()>,
    {
        match self.token_map.write().await.get_mut(token) {
            Some(player) => {
                let mut player = player.write().await;
                match handler(&mut *player) {
                    Some(()) => Ok(()),
                    None => Err(()),
                }
            }
            None => Err(()),
        }
    }
}
