use super::depends::*;

use crate::{constants::PresenceFilter, objects::PlayMods, packets};
use num_traits::FromPrimitive;

#[inline(always)]
/// Player logout from server
pub async fn user_logout<'a>(ctx: &HandlerContext<'a>) {
    ctx.player_sessions
        .write()
        .await
        .logout(ctx.token, Some(ctx.channel_list))
        .await;
}

#[inline(always)]
/// Update player's presence_filter
pub async fn receive_updates<'a>(ctx: &HandlerContext<'a>) {
    let filter_val = PayloadReader::new(ctx.payload).read_integer::<i32>().await;
    match ctx
        .player_sessions
        .read()
        .await
        .handle_player(ctx.token, move |p| {
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
#[inline(always)]
/// #85: OSU_USER_STATS_REQUEST
/// 
/// Send other's stats to self
pub async fn stats_request<'a>(ctx: &HandlerContext<'a>) {
    let id_list = PayloadReader::new(ctx.payload).read_i32_list::<i16>().await;

    let player_sessions = ctx.player_sessions.read().await;
    let id_session_map = player_sessions.id_session_map.read().await;

    if let Some(ctx_player) = id_session_map.get(&ctx.id) {
        let ctx_player = ctx_player.read().await;

        for player_id in &id_list {
            // Skip self
            if *player_id == ctx.id {
                continue;
            }

            if let Some(player) = id_session_map.get(player_id) {
                ctx_player
                    .enqueue(packets::user_stats(&*player.read().await).await)
                    .await;
            }
        }
    }
}

#[inline(always)]
/// #97: OSU_USER_PRESENCE_REQUEST
/// 
/// Send other's presence to self (list)
pub async fn presence_request<'a>(ctx: &HandlerContext<'a>) {
    let id_list = PayloadReader::new(ctx.payload).read_i32_list::<i16>().await;

    let player_sessions = ctx.player_sessions.read().await;
    let id_session_map = player_sessions.id_session_map.read().await;

    if let Some(ctx_player) = id_session_map.get(&ctx.id) {
        let ctx_player = ctx_player.read().await;

        for player_id in &id_list {
            if let Some(player) = id_session_map.get(player_id) {
                // Send presence to self
                ctx_player
                    .enqueue(packets::user_presence(&*player.read().await).await)
                    .await;
            }
        }
    }
}

#[inline(always)]
/// # 98: OSU_USER_PRESENCE_REQUEST_ALL (non-payload)
/// 
// Send other's presence to self (all)
pub async fn presence_request_all<'a>(ctx: &HandlerContext<'a>) {
    let player_sessions = ctx.player_sessions.read().await;
    let id_session_map = player_sessions.id_session_map.read().await;

    if let Some(ctx_player) = id_session_map.get(&ctx.id) {
        let ctx_player = ctx_player.read().await;

        for (player_id, player) in id_session_map.iter() {
            // Skip self
            if *player_id == ctx.id {
                continue;
            }
            // Send presence to self
            ctx_player
                .enqueue(packets::user_presence(&*player.read().await).await)
            .await;
    }
}
}

#[inline(always)]
/// Update player's status
pub async fn change_action<'a>(ctx: &HandlerContext<'a>) {
    // Read the packet
    let mut reader = PayloadReader::new(ctx.payload);
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
                ctx.name, ctx.id, action
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
                ctx.name, ctx.id, game_mode, play_mod_list
            );
            return;
        }
    };

    debug!(
        "Player {}({}) changing action: <a: {:?} i: {} b: {} pm: {:?} gm: {:?} bid: {}>",
        ctx.name,
        ctx.id,
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
    let update_stats = game_mode != ctx.data.status.game_mode;
    let (stats, should_update_cache) = match update_stats {
        true => {
            // Switch to new game mod stats!
            ctx.data.get_stats_update(game_mode, ctx.database).await
        }
        false => (None, false),
    };

    // Update player's status and send it to all players.
    // Get lock first.
    let player_sessions = ctx.player_sessions.read().await;

    match player_sessions
        .handle_player_get(ctx.token, move |p| {
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
                ctx.name, ctx.id,
            )
        }
    };
}

#[inline(always)]
/// Add a player to friends
pub async fn add_friend<'a>(ctx: &HandlerContext<'a>) {
    let target_id = PayloadReader::new(ctx.payload).read_integer::<i32>().await;
    handle_add_friend(target_id, ctx).await;
}

#[inline(always)]
/// Add a player to friends
pub async fn handle_add_friend<'a>(target_id: i32, ctx: &HandlerContext<'a>) {
    // -1 is BanchoBot, not exists
    if target_id == -1 {
        return;
    }

    // Add an offline player is not allowed
    if !ctx
        .player_sessions
        .read()
        .await
        .id_is_exists(&target_id)
        .await
    {
        warn!(
            "Player {}({}) tries to add an offline user {} to friends.",
            ctx.name, ctx.id, target_id
        );
        return;
    };

    // Add friend in server
    let result = ctx
        .player_sessions
        .read()
        .await
        .handle_player(ctx.token, |p| {
            if p.friends.contains(&target_id) {
                return None;
            }
            p.friends.push(target_id);
            Some(())
        })
        .await;

    if !result.is_ok() {
        info!(
            "Player {}({}) already added {} to friends.",
            ctx.name, ctx.id, target_id
        );
        return;
    };

    // Add friend in database
    if let Err(err) = ctx
        .database
        .pg
        .execute(
            r#"INSERT INTO "user"."friends" VALUES ($1, $2);"#,
            &[&ctx.id, &target_id],
        )
        .await
    {
        error!(
            "Failed to add friend {} for player {}({}), error: {:?}",
            target_id, ctx.name, ctx.id, err
        );
        return;
    }

    info!(
        "Player {}({}) added {} to friends.",
        ctx.name, ctx.id, target_id
    );
}

#[inline(always)]
/// Remove a player from friends
pub async fn remove_friend<'a>(ctx: &HandlerContext<'a>) {
    let target = PayloadReader::new(ctx.payload).read_integer::<i32>().await;
    handle_remove_friend(target, ctx).await;
}

#[inline(always)]
/// Remove a player from friends
pub async fn handle_remove_friend<'a>(target: i32, ctx: &HandlerContext<'a>) {
    // -1 is BanchoBot, not exists
    if target == -1 {
        return;
    }

    // Remove a offline player is not allowed
    if !ctx.player_sessions.read().await.id_is_exists(&target).await {
        info!(
            "Player {}({}) tries to remove a offline {} from friends.",
            ctx.name, ctx.id, target
        );
        return;
    };

    // Remove friend in server
    let result = ctx
        .player_sessions
        .read()
        .await
        .handle_player(ctx.token, |p| {
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
            ctx.name, ctx.id, target
        );
        return;
    };

    // Remove friend from database
    if let Err(err) = ctx
        .database
        .pg
        .execute(
            r#"DELETE FROM "user"."friends" WHERE "user_id" = $1 AND "friend_id" = $2;"#,
            &[&ctx.id, &target],
        )
        .await
    {
        error!(
            "Failed to remove friend {} from player {}({}), error: {:?}",
            target, ctx.name, ctx.id, err
        );
        return;
    }

    info!(
        "Player {}({}) removed {} from friends.",
        ctx.name, ctx.id, target
    );
}

#[inline(always)]
/// Player toggle block-non-friend-dms with a value
pub async fn toggle_block_non_friend_dms<'a>(ctx: &HandlerContext<'a>) {
    let value = PayloadReader::new(ctx.payload).read_integer::<i32>().await;
    match ctx
        .player_sessions
        .read()
        .await
        .handle_player(ctx.token, |p| {
            p.only_friend_pm_allowed = value == 1;
            Some(())
        })
        .await
    {
        Ok(()) => {
            debug!(
                "Player {}({}) toggled block-non-friend-dms with value {}",
                ctx.name, ctx.id, value
            );
        }
        Err(()) => {
            error!(
                "Player {}({}) failed to toggle block-non-friend-dms with value {}",
                ctx.name, ctx.id, value
            );
        }
    }
}

#[inline(always)]
/// Player join to a channel
pub async fn channel_join<'a>(ctx: &HandlerContext<'a>) {
    let channel_name = PayloadReader::new(ctx.payload).read_string().await;
    match ctx.channel_list.read().await.get(&channel_name) {
        Some(channel) => {
            channel.join(ctx.id, None).await;
        }
        None => {
            debug!(
                "Player {}({}) try join to a non-exists channel {}!",
                ctx.name, ctx.id, channel_name
            );
        }
    };
}

#[inline(always)]
/// Player leave from a channel
pub async fn channel_part<'a>(ctx: &HandlerContext<'a>) {
    let channel_name = PayloadReader::new(ctx.payload).read_string().await;
    match ctx.channel_list.read().await.get(&channel_name) {
        Some(channel) => {
            channel.leave(ctx.id, None).await;
        }
        None => {
            debug!(
                "Player {}({}) try to part from a non-exists channel {}!",
                ctx.name, ctx.id, channel_name
            );
        }
    };
}

#[inline(always)]
/// #82: OSU_USER_SET_AWAY_MESSAGE
/// 
pub async fn set_away_message<'a>(ctx: &HandlerContext<'a>) {
    let message = PayloadReader::new(ctx.payload).read_message().await;
    let player_sessions = ctx.player_sessions.read().await;
    let mut id_session_map = player_sessions.id_session_map.write().await;
    if let Some(player) = id_session_map.get_mut(&ctx.id) {
        player.write().await.away_message = message.content;
    };
}
