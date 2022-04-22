use super::depends::*;
use bancho_packets::{PayloadReader, BanchoMessage};
use chrono::Local;
use num_traits::FromPrimitive;
use peace_constants::{PlayMods, PresenceFilter};

#[inline]
/// #2: OSU_USER_LOGOUT
///
/// Player logout from server
pub async fn user_logout<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    if (Local::now().timestamp() - ctx.data.login_time.timestamp()) < 1 {
        return None;
    }

    write_lock!(ctx.bancho.player_sessions)
        .logout(ctx.token, Some(&ctx.bancho.channel_list))
        .await;
    Some(())
}

#[inline]
/// #79: OSU_USER_RECEIVE_UPDATES
///
/// Update player's presence_filter
pub async fn receive_updates<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let filter_val = PayloadReader::new(ctx.payload).read::<i32>()?;
    let filter: Option<PresenceFilter> = PresenceFilter::from_i32(filter_val);

    if filter.is_none() {
        error!("Failed to update player {}({})'s presence filter, invaild filter value {}! <OSU_USER_RECEIVE_UPDATES>", ctx.name, ctx.id, filter_val);
        return None;
    }

    if let Some(player) = ctx.weak_player.upgrade() {
        write_lock!(player).presence_filter = filter.unwrap();
    };
    Some(())
}

#[inline]
/// #3: OSU_USER_REQUEST_STATUS_UPDATE (non-payload)
///
/// Update self's status for self
pub async fn request_status_update<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    if let Some(player) = ctx.weak_player.upgrade() {
        let player = read_lock!(player);
        player.enqueue(player.stats_packet()).await;
    };
    Some(())
}

#[inline]
/// #85: OSU_USER_STATS_REQUEST
///
/// Send other's stats to self
pub async fn stats_request<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let id_list = PayloadReader::new(ctx.payload).read::<Vec<i32>>()?;

    let player_sessions = read_lock!(ctx.bancho.player_sessions);
    let id_map = &player_sessions.id_map;

    if let Some(ctx_player) = &ctx.weak_player.upgrade() {
        let ctx_player = read_lock!(ctx_player);

        for player_id in &id_list {
            // Skip self
            if *player_id == ctx.id {
                continue;
            }

            if let Some(player) = id_map.get(player_id) {
                let p = read_lock!(player).stats_packet();
                ctx_player.enqueue(p).await;
            }
        }
    };
    Some(())
}

#[inline]
/// #97: OSU_USER_PRESENCE_REQUEST
///
/// Send other's presence to self (list)
pub async fn presence_request<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let id_list = PayloadReader::new(ctx.payload).read::<Vec<i32>>()?;
    let mut user_presence_packets: Vec<Vec<u8>> = Vec::with_capacity(id_list.len());

    {
        let player_sessions = read_lock!(ctx.bancho.player_sessions);
        for player_id in &id_list {
            if let Some(player) = player_sessions.id_map.get(player_id) {
                let others = read_lock!(player);
                user_presence_packets
                    .push(others.presence_packet(ctx.data.settings.display_u_name));
            }
        }
        drop(player_sessions);
    }

    if let Some(ctx_player) = ctx.weak_player.upgrade() {
        let ctx_player = read_lock!(ctx_player);
        for packets in user_presence_packets {
            ctx_player.enqueue(packets).await;
        }
    };
    Some(())
}

#[inline]
/// # 98: OSU_USER_PRESENCE_REQUEST_ALL (non-payload)
///
// Send other's presence to self (all)
pub async fn presence_request_all<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let user_presence_packets = {
        let player_sessions = read_lock!(ctx.bancho.player_sessions);
        let id_map = &player_sessions.id_map;

        let mut user_presence_packets: Vec<Vec<u8>> = Vec::with_capacity(id_map.len());

        for (player_id, player) in player_sessions.id_map.iter() {
            // Skip self
            if *player_id == ctx.id {
                continue;
            }
            // Send presence to self
            let others = read_lock!(player);
            user_presence_packets.push(others.presence_packet(ctx.data.settings.display_u_name));
        }

        drop(player_sessions);

        user_presence_packets
    };

    if let Some(ctx_player) = ctx.weak_player.upgrade() {
        let ctx_player = read_lock!(ctx_player);
        for packets in user_presence_packets {
            ctx_player.enqueue(packets).await;
        }
    };
    Some(())
}

#[inline]
/// #0: OSU_USER_CHANGE_ACTION
///
/// Update player's status
pub async fn change_action<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    // Read the packet
    let mut reader = PayloadReader::new(ctx.payload);
    let (action, info, beatmap_md5, play_mods_value, game_mode_u8, beatmap_id) = (
        reader.read::<u8>()?,
        reader.read()?,
        reader.read()?,
        reader.read::<u32>()?,
        reader.read::<u8>()?,
        reader.read::<i32>()?,
    );

    let action = match Action::from_u8(action) {
        Some(action) => action,
        None => {
            error!(
                "Failed to parse player {}({})'s action({})! <OSU_CHANGE_ACTION>",
                ctx.name, ctx.id, action
            );
            return None;
        }
    };

    let playmod_list = PlayMods::get_mods(play_mods_value);
    let game_mode = GameMode::parse_with_playmod(game_mode_u8, &playmod_list);
    if game_mode.is_none() {
        error!(
            "Failed to parse player {}({})'s game mode({:?})! <OSU_CHANGE_ACTION>; play_mod_list: {:?}",
            ctx.name, ctx.id, game_mode_u8, playmod_list
        );
        return None;
    }
    let game_mode = game_mode.unwrap();

    debug!(
        "Player {}({}) changing action: <a: {:?} i: {} b: {} pm: {:?} gm: {:?} bid: {}>",
        ctx.name, ctx.id, action, info, beatmap_md5, playmod_list, game_mode, beatmap_id
    );

    // Update player's status
    let player_stats_packets_new = if let Some(player) = ctx.weak_player.upgrade() {
        let mut player = write_lock!(player);
        player.update_status(
            action,
            info,
            beatmap_md5,
            beatmap_id,
            play_mods_value,
            game_mode,
        );
        player.update_stats(ctx.database).await;
        player.stats_packet()
    } else {
        error!(
            "Failed to update player {}({})'s status! <OSU_CHANGE_ACTION>",
            ctx.name, ctx.id,
        );
        return None;
    };

    // Send it to all players
    read_lock!(ctx.bancho.player_sessions)
        .enqueue_all(&player_stats_packets_new)
        .await;
    Some(())
}

#[inline]
/// #73: OSU_USER_FRIEND_ADD
///
/// Add a player to friends
pub async fn add_friend<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let target_id = PayloadReader::new(ctx.payload).read::<i32>()?;

    // -1 is BanchoBot, not exists
    if target_id == -1 {
        return None;
    }

    // Add an offline player is not allowed
    if !read_lock!(ctx.bancho.player_sessions)
        .id_is_exists(&target_id)
        .await
    {
        warn!(
            "Player {}({}) tries to add an offline user {} to friends.",
            ctx.name, ctx.id, target_id
        );
        return None;
    };

    handle_add_friend(target_id, ctx).await;
    Some(())
}

#[inline]
/// Add a player to friends
pub async fn handle_add_friend<'a>(target_id: i32, ctx: &HandlerContext<'a>) -> Option<()> {
    // Try get player
    let player = ctx.weak_player.upgrade()?;

    {
        let mut player = write_lock!(player.as_ref());

        if player.friends.contains(&target_id) {
            info!(
                "Player {}({}) already added {} to friends.",
                ctx.name, ctx.id, target_id
            );
            return None;
        };

        // Add friend in server
        player.friends.push(target_id);

        drop(player);
    }

    // Add friend in database
    if let Err(err) = ctx
        .database
        .pg
        .execute(
            r#"INSERT INTO "user"."friends" VALUES ($1, $2) ON CONFLICT DO NOTHING;"#,
            &[&ctx.id, &target_id],
        )
        .await
    {
        error!(
            "Failed to add friend {} for player {}({}), error: {:?}",
            target_id, ctx.name, ctx.id, err
        );
        return None;
    }

    info!(
        "Player {}({}) added {} to friends.",
        ctx.name, ctx.id, target_id
    );
    Some(())
}

#[inline]
/// #74: OSU_USER_FRIEND_REMOVE
///
/// Remove a player from friends
pub async fn remove_friend<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let target = PayloadReader::new(ctx.payload).read::<i32>()?;

    // -1 is BanchoBot, not exists
    if target == -1 {
        return None;
    }

    // Remove a offline player is not allowed
    if !read_lock!(ctx.bancho.player_sessions)
        .id_is_exists(&target)
        .await
    {
        info!(
            "Player {}({}) tries to remove a offline {} from friends.",
            ctx.name, ctx.id, target
        );
        return None;
    };
    handle_remove_friend(target, ctx).await;
    Some(())
}

#[inline]
/// Remove a player from friends
pub async fn handle_remove_friend<'a>(target: i32, ctx: &HandlerContext<'a>) -> Option<()> {
    // Try get player
    let player = ctx.weak_player.upgrade()?;

    {
        let mut player = write_lock!(player.as_ref());

        let friend_index = player.friends.binary_search(&target);
        if friend_index.is_err() {
            info!(
                "Player {}({}) already removed {} from friends.",
                ctx.name, ctx.id, target
            );
            return None;
        };

        // Remove friend in server
        player.friends.remove(friend_index.unwrap());

        drop(player);
    }

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
        return None;
    }

    info!(
        "Player {}({}) removed {} from friends.",
        ctx.name, ctx.id, target
    );
    Some(())
}

#[inline]
/// #99: OSU_USER_TOGGLE_BLOCK_NON_FRIEND_DMS
///
/// Player toggle block-non-friend-dms with a value
pub async fn toggle_block_non_friend_dms<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let value = PayloadReader::new(ctx.payload).read::<i32>()?;

    if let Some(player) = ctx.weak_player.upgrade() {
        write_lock!(player).only_friend_pm_allowed = value == 1;
        debug!(
            "Player {}({}) toggled block-non-friend-dms with value {}",
            ctx.name, ctx.id, value
        );
    };
    Some(())
}

#[inline]
/// #63: OSU_USER_CHANNEL_JOIN
///
/// Player join to a channel
pub async fn channel_join<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let channel_name = PayloadReader::new(ctx.payload).read::<String>()?;

    if channel_name == "#highlight" {
        return Some(());
    }

    match read_lock!(ctx.bancho.channel_list).get(&channel_name) {
        Some(channel) => {
            let mut c = write_lock!(channel);
            if c.auto_close {
                c.join_player(ctx.weak_player.upgrade()?, None).await
            } else {
                let s = read_lock!(ctx.bancho.player_sessions);
                c.join_player(ctx.weak_player.upgrade()?, Some(&*s)).await
            };
        }
        None => {
            debug!(
                "Player {}({}) try join to a non-exists channel {}!",
                ctx.name, ctx.id, channel_name
            );
        }
    };
    Some(())
}

#[inline]
/// #78: OSU_USER_CHANNEL_PART
///
/// Player leave from a channel
pub async fn channel_part<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let channel_name = PayloadReader::new(ctx.payload).read::<String>()?;
    match read_lock!(ctx.bancho.channel_list).get(&channel_name) {
        Some(channel) => {
            let mut c = write_lock!(channel);
            if c.auto_close {
                c.leave(ctx.id, None).await
            } else {
                let s = read_lock!(ctx.bancho.player_sessions);
                c.leave(ctx.id, Some(&*s)).await
            };
        }
        None => {
            debug!(
                "Player {}({}) try to part from a non-exists channel {}!",
                ctx.name, ctx.id, channel_name
            );
        }
    };
    Some(())
}

#[inline]
/// #82: OSU_USER_SET_AWAY_MESSAGE
///
pub async fn set_away_message<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let mut message = PayloadReader::new(ctx.payload).read::<BanchoMessage>()?;

    let cfg_r = read_lock!(ctx.bancho.config);
    let cfg = &cfg_r.data;

    // Limit the length of message content
    if let Some(max_len) = cfg.message.max_length {
        let max_len = max_len as usize;
        if message.content.len() > max_len {
            message.content = message.content[0..max_len].to_string();
        }
    };

    // sensitive words replace
    for i in &cfg.server.sensitive_words {
        message.content = message.content.replace(i, "**")
    }

    if let Some(player) = ctx.weak_player.upgrade() {
        write_lock!(player).away_message = message.content;
    };
    Some(())
}

#[inline]
/// #29: OSU_USER_PART_LOBBY
///
pub async fn lobby_part<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    // Try get player
    let player = ctx.weak_player.upgrade()?;
    {
        let mut player = write_lock!(player.as_ref());
        player.in_lobby = false;
        drop(player);
    }
    Some(())
}

#[inline]
/// #30: OSU_USER_JOIN_LOBBY
///
pub async fn lobby_join<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    // Try get player
    let player = ctx.weak_player.upgrade()?;
    {
        let mut player = write_lock!(player.as_ref());
        player.in_lobby = true;
        drop(player);
    }
    Some(())
}
