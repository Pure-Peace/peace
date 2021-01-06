use super::depends::*;

use crate::{constants::PresenceFilter, objects::PlayMods, packets};
use num_traits::FromPrimitive;

#[inline(always)]
/// Player logout from server
pub async fn user_logout(
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    channel_list: &Data<RwLock<ChannelList>>,
) {
    player_sessions
        .write()
        .await
        .logout(token, Some(channel_list))
        .await;
}

#[inline(always)]
/// Update player's presence_filter
pub async fn receive_updates(
    payload: &[u8],
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let filter_val = PayloadReader::new(&payload).read_integer::<i32>().await;
    match player_sessions
        .write()
        .await
        .handle_player(token, move |p| {
            p.presence_filter = PresenceFilter::from_i32(filter_val)?;
            Some(())
        })
        .await
    {
        Ok(()) => {}
        Err(()) => {
            error!("Failed to update player's presence filter! <OSU_USER_RECEIVE_UPDATES>");
        }
    }
}

#[inline(always)]
/// Send the player stats by requests
pub async fn stats_request(
    payload: &[u8],
    player_sessions: &Data<RwLock<PlayerSessions>>,
    player_data: &PlayerData,
) {
    let id_list = PayloadReader::new(&payload).read_i32_list::<i16>().await;
    let player_sessions = player_sessions.read().await;
    for p_id in &id_list {
        player_sessions
            .enqueue_by_id(p_id, packets::user_presence_from_data(&player_data).await)
            .await;
    }
}

#[inline(always)]
/// Update player's status
pub async fn change_action(
    payload: &[u8],
    database: &Database,
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    player_data: &PlayerData,
) {
    // Read the packet
    let mut reader = PayloadReader::new(&payload);
    let (action, info, playing_beatmap_md5, play_mods_value, mut game_mode, playing_beatmap_id) = (
        reader.read_integer::<u8>().await,
        reader.read_string().await,
        reader.read_string().await,
        reader.read_integer::<u32>().await,
        reader.read_integer::<u8>().await,
        reader.read_integer::<i32>().await,
    );

    let action = match Action::from_u8(action) {
        Some(action) => action,
        None => {
            error!(
                "Failed to parse player {}({})'s action({})! <OSU_CHANGE_ACTION>",
                player_data.name, player_data.id, action
            );
            return;
        }
    };

    let play_mod_list = PlayMods::get_mods(play_mods_value);

    // !More detailed game mod but:
    //
    // 1. Mania have not relax
    // 2. only std have autopilot
    // 3. relax and autopilot cannot coexist
    //
    if game_mode != 3 && play_mod_list.contains(&PlayMod::Relax) {
        game_mode += 4;
    } else if game_mode == 0 && play_mod_list.contains(&PlayMod::AutoPilot) {
        game_mode += 8;
    }

    let game_mode = match GameMode::from_u8(game_mode) {
        Some(action) => action,
        None => {
            error!(
                "Failed to parse player {}({})'s game mode({})! <OSU_CHANGE_ACTION>; play_mod_list: {:?}",
                player_data.name, player_data.id, game_mode, play_mod_list
            );
            return;
        }
    };

    debug!(
        "Player {}({}) changing action: <a: {:?} i: {} b: {} pm: {:?} gm: {:?} bid: {}>",
        player_data.name,
        player_data.id,
        action,
        info,
        playing_beatmap_md5,
        play_mod_list,
        game_mode,
        playing_beatmap_id
    );

    // Should update stats and rank or not
    //
    // Why am I using player_data instead of player for this?
    // Because I want to reduce the length of time the lock is used (calculate rank from database will consume some time)
    //
    let update_stats = game_mode != player_data.status.game_mode;
    let (stats, should_update_cache) = match update_stats {
        true => {
            // Switch to new game mod stats!
            player_data.get_stats_update(game_mode, database).await
        }
        false => (None, false),
    };

    // Update player's status and send it to all players.
    // Get lock first.
    let player_sessions = player_sessions.read().await;

    match player_sessions
        .handle_player_get(token, move |p| {
            if update_stats && stats.is_some() {
                // Update cache if we should
                if should_update_cache {
                    p.stats_cache.insert(game_mode, stats.clone().unwrap());
                }
                // Update stats
                p.stats = stats.unwrap();
            };
            p.update_status(
                action,
                info,
                playing_beatmap_md5,
                playing_beatmap_id,
                play_mods_value,
                game_mode,
            )
        })
        .await
    {
        Ok(player_data) => {
            player_sessions
                .enqueue_all(&packets::user_stats_from_data(&player_data).await)
                .await
        }
        Err(()) => {
            error!(
                "Failed to update player {}({})'s status! <OSU_CHANGE_ACTION>",
                player_data.name, player_data.id,
            )
        }
    };
}

#[inline(always)]
/// Add a player to friends
pub async fn add_friend(
    payload: &[u8],
    database: &Database,
    player_data: &PlayerData,
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let target = PayloadReader::new(&payload).read_integer::<i32>().await;

    // -1 is BanchoBot, not exists
    if target == -1 {
        return;
    }

    // Add a offline player is not allowed
    if !player_sessions.read().await.id_is_exists(&target).await {
        info!(
            "Player {}({}) tries to add a offline {} to friends.",
            player_data.name, player_data.id, target
        );
        return;
    };

    // Add friend in server
    let result = player_sessions
        .write()
        .await
        .handle_player(&token, |p| {
            if p.friends.contains(&target) {
                return None;
            }
            p.friends.push(target);
            Some(())
        })
        .await;

    if !result.is_ok() {
        info!(
            "Player {}({}) already added {} to friends.",
            player_data.name, player_data.id, target
        );
        return;
    };

    // Add friend in database
    if let Err(err) = database
        .pg
        .execute(
            r#"INSERT INTO "user"."friends" VALUES ($1, $2);"#,
            &[&player_data.id, &target],
        )
        .await
    {
        error!(
            "Failed to add friend {} for player {}({}), error: {:?}",
            target, player_data.name, player_data.id, err
        );
        return;
    }

    info!(
        "Player {}({}) added {} to friends.",
        player_data.name, player_data.id, target
    );
}

#[inline(always)]
/// Remove a player from friends
pub async fn remove_friend(
    payload: &[u8],
    database: &Database,
    player_data: &PlayerData,
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let target = PayloadReader::new(&payload).read_integer::<i32>().await;

    // -1 is BanchoBot, not exists
    if target == -1 {
        return;
    }

    // Remove a offline player is not allowed
    if !player_sessions.read().await.id_is_exists(&target).await {
        info!(
            "Player {}({}) tries to remove a offline {} from friends.",
            player_data.name, player_data.id, target
        );
        return;
    };

    // Remove friend in server
    let result = player_sessions
        .write()
        .await
        .handle_player(&token, |p| {
            if let Ok(idx) = p.friends.binary_search(&target) {
                p.friends.remove(idx);
                return Some(());
            }
            None
        })
        .await;

    if !result.is_ok() {
        info!(
            "Player {}({}) already removed {} from friends.",
            player_data.name, player_data.id, target
        );
        return;
    };

    // Remove friend from database
    if let Err(err) = database
        .pg
        .execute(
            r#"DELETE FROM "user"."friends" WHERE "user_id" = $1 AND "friend_id" = $2;"#,
            &[&player_data.id, &target],
        )
        .await
    {
        error!(
            "Failed to remove friend {} from player {}({}), error: {:?}",
            target, player_data.name, player_data.id, err
        );
        return;
    }

    info!(
        "Player {}({}) removed {} from friends.",
        player_data.name, player_data.id, target
    );
}

#[inline(always)]
/// Player toggle block-non-friend-dms with a value
pub async fn toggle_block_non_friend_dms(
    payload: &[u8],
    token: &String,
    player_data: &PlayerData,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let value = PayloadReader::new(&payload).read_integer::<i32>().await;
    match player_sessions
        .read()
        .await
        .handle_player(&token, |p| {
            p.only_friend_pm_allowed = value == 1;
            Some(())
        })
        .await
    {
        Ok(()) => {
            debug!(
                "Player {}({}) toggled block-non-friend-dms with value {}",
                player_data.name, player_data.id, value
            );
        }
        Err(()) => {
            error!(
                "Player {}({}) failed to toggle block-non-friend-dms with value {}",
                player_data.name, player_data.id, value
            );
        }
    }
}

#[inline(always)]
/// Player leave from a channel
pub async fn channel_part(
    payload: &[u8],
    token: &String,
    player_data: &PlayerData,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    channel_list: &Data<RwLock<ChannelList>>,
) {
    let channel_name = PayloadReader::new(&payload).read_string().await;

    match channel_list.write().await.get_mut(&channel_name) {
        Some(channel) => {
            let player_sessions = player_sessions.read().await;

            {
                let mut map = player_sessions.map.write().await;
                let player = map.get_mut(token);
                if player.is_none() {
                    return;
                }
                channel.leave(player.unwrap()).await;
                drop(map);
            }

            channel.update_channel_for_users(&player_sessions).await;
        }
        None => {
            error!(
                "Player {}({}) try to part from a non-exists channel {}!",
                player_data.name, player_data.id, channel_name
            );
        }
    };
}

#[inline(always)]
/// Player join to a channel
pub async fn channel_join(
    payload: &[u8],
    token: &String,
    player_data: &PlayerData,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    channel_list: &Data<RwLock<ChannelList>>,
) {
    let channel_name = PayloadReader::new(&payload).read_string().await;

    match channel_list.write().await.get_mut(&channel_name) {
        Some(channel) => {
            let player_sessions = player_sessions.read().await;

            {
                let mut map = player_sessions.map.write().await;
                let player = map.get_mut(token);
                if player.is_none() {
                    return;
                }
                channel.join(player.unwrap()).await;
                drop(map);
            }

            channel.update_channel_for_users(&player_sessions).await;
        }
        None => {
            error!(
                "Player {}({}) try join to a non-exists channel {}!",
                player_data.name, player_data.id, channel_name
            );
        }
    };
}
