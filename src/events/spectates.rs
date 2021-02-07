use chrono::Local;
use hashbrown::HashSet;

use crate::{objects::Channel, packets};

use super::depends::*;
use super::users;

pub async fn spectate_start<'a>(ctx: &HandlerContext<'a>) {
    let target_id = PayloadReader::new(ctx.payload).read_integer::<i32>().await;

    // -1 is BanchoBot, not exists
    if target_id == -1 {
        return;
    }

    // Specate an offline player is not allowed
    let target_token = match ctx
        .player_sessions
        .read()
        .await
        .get_token_by_id(&target_id)
        .await
    {
        Some(token) => token,
        None => {
            warn!(
                "Player {}({}) tries to spectate an offline user {}.",
                ctx.name, ctx.id, target_id
            );
            return;
        }
    };

    // If already spectating someone
    if let Some(spectating_id) = ctx.data.spectating {
        // Then remove spectating from me
        let _unused_result = ctx
            .player_sessions
            .write()
            .await
            .handle_player(ctx.token, |p| {
                p.spectating = None;
                Some(())
            })
            .await;

        // Try part me from spectating channel
        let spectating_channel = format!("#spect_{}", spectating_id);
        users::handle_channel_part(&spectating_channel, ctx, None).await;

        // Get spectating player id
        let spectating_token = match ctx
            .player_sessions
            .read()
            .await
            .get_token_by_id(&spectating_id)
            .await
        {
            Some(token) => token,
            None => {
                error!("Failed to get spectating token {}.", spectating_id);
                return;
            }
        };

        // Try remove me from spectating player
        let spectating = match ctx
            .player_sessions
            .write()
            .await
            .handle_player_get(&spectating_token, |p| {
                p.spectators.remove(&ctx.id);
                Some(())
            })
            .await
        {
            Ok(spectating) => spectating,
            Err(()) => {
                error!("Failed to stop spectating {}.", spectating_id);
                return;
            }
        };

        // If not any one spectate
        if spectating.spectators.len() == 0 {
            users::handle_channel_part(&spectating_channel, ctx, Some(&spectating_token)).await;
        } else {
            let fellow = packets::fellow_spectator_left(ctx.id);
            let channel_info = {
                let c_lock = ctx.channel_list.read().await;
                match c_lock.get(&spectating_channel) {
                    Some(c) => packets::channel_info(&c.name, &c.title, c.player_count),
                    None => packets::channel_info(&spectating_channel, &spectating_channel, 99),
                }
            };

            {
                let player_sessions = ctx.player_sessions.write().await;
                let mut player_sessions_map = player_sessions.token_map.write().await;
                if let Some(p) = player_sessions_map.get_mut(&spectating_token) {
                    p.read().await.enqueue(channel_info.clone()).await;
                };

                let id_session_map = player_sessions.id_session_map.read().await;
                for id in spectating.spectators {
                    if let Some(p) = id_session_map.get(&id) {
                        let p = p.read().await;
                        p.enqueue(fellow.clone()).await;
                        p.enqueue(channel_info.clone()).await;
                    };
                }
            }
        };

        {
            let player_sessions = ctx.player_sessions.write().await;
            let mut player_sessions_map = player_sessions.token_map.write().await;
            if let Some(p) = player_sessions_map.get_mut(&spectating_token) {
                p.read()
                    .await
                    .enqueue(packets::spectator_left(ctx.id))
                    .await;
            };
            info!("{} is no longer watching {}.", ctx.name, spectating.name)
        }
    };

    // Create channel if not exists
    let channel_name = format!("#spect_{}", target_id);
    if ctx.channel_list.read().await.contains_key(&channel_name) == false {
        let channel = Channel {
            name: channel_name.clone(),
            title: channel_name.clone(),
            read_priv: 1,
            write_priv: 1,
            auto_join: false,
            auto_close: true,
            players: RwLock::new(HashSet::new()),
            player_count: 0,
            create_time: Local::now(),
        };
        ctx.channel_list
            .write()
            .await
            .insert(channel_name.clone(), channel);
        info!("Spectate channel {} created.", channel_name);
    };

    // Join to channel
    users::handle_channel_join(&channel_name, ctx, None).await;

    let p_joined = packets::fellow_spectator_joined(ctx.id);

    let player_sessions = ctx.player_sessions.read().await;
    let player_id_session_map = player_sessions.id_session_map.read().await;

    let mut spectators = vec![];
    for id in player_sessions
        .get_player_data(&target_token)
        .await
        .unwrap()
        .spectators
        .iter()
    {
        if let Some(player) = player_id_session_map.get(id) {
            spectators.push(player)
        }
    }

    let spectators1: Vec<i32> = player_sessions
        .get_player_data(&target_token)
        .await
        .unwrap()
        .spectators
        .iter()
        .map(|id| id.clone())
        .collect();

    let player_sessions = ctx.player_sessions.write().await;
    for pp in spectators {
        pp.read().await.enqueue(p_joined.clone()).await;
    }

    let mut player_sessions_map = player_sessions.token_map.write().await;

    {
        let mut me = player_sessions_map
            .get_mut(ctx.token)
            .unwrap()
            .write()
            .await;
        for s in spectators1 {
            me.enqueue(packets::fellow_spectator_joined(s)).await;
        }
        me.spectating = Some(target_id);
    }

    let mut target = player_sessions_map
        .get_mut(&target_token)
        .unwrap()
        .write()
        .await;
    target.spectators.insert(ctx.id);
    target.enqueue(packets::spectator_joined(ctx.id)).await;
    info!("fuck!");
}

pub async fn spectate_stop<'a>(ctx: &HandlerContext<'a>) {
    let spectating_id = ctx.data.spectating;
    if spectating_id.is_none() {
        error!("sb");
        return;
    }
    let spectating_id = spectating_id.unwrap();

    // Then remove spectating from me
    let _unused_result = ctx
        .player_sessions
        .write()
        .await
        .handle_player(ctx.token, |p| {
            p.spectating = None;
            Some(())
        })
        .await;

    // Try part me from spectating channel
    let spectating_channel = format!("#spect_{}", spectating_id);
    users::handle_channel_part(&spectating_channel, ctx, None).await;

    // Get spectating player id
    let spectating_token = match ctx
        .player_sessions
        .read()
        .await
        .get_token_by_id(&spectating_id)
        .await
    {
        Some(token) => token,
        None => {
            error!("Failed to get spectating token {}.", spectating_id);
            return;
        }
    };

    // Try remove me from spectating player
    let spectating = match ctx
        .player_sessions
        .write()
        .await
        .handle_player_get(&spectating_token, |p| {
            p.spectators.remove(&ctx.id);
            Some(())
        })
        .await
    {
        Ok(spectating) => spectating,
        Err(()) => {
            error!("Failed to stop spectating {}.", spectating_id);
            return;
        }
    };

    // If not any one spectate
    if spectating.spectators.len() == 0 {
        users::handle_channel_part(&spectating_channel, ctx, Some(&spectating_token)).await;
    } else {
        let fellow = packets::fellow_spectator_left(ctx.id);
        let channel_info = {
            let c_lock = ctx.channel_list.read().await;
            match c_lock.get(&spectating_channel) {
                Some(c) => packets::channel_info(&c.name, &c.title, c.player_count),
                None => packets::channel_info(&spectating_channel, &spectating_channel, 99),
            }
        };

        {
            let player_sessions = ctx.player_sessions.write().await;
            let player_sessions_map = player_sessions.token_map.read().await;
            if let Some(p) = player_sessions_map.get(&spectating_token) {
                p.read().await.enqueue(channel_info.clone()).await;
            };

            let id_session_map = player_sessions.id_session_map.read().await;
            for id in spectating.spectators {
                if let Some(p) = id_session_map.get(&id) {
                    let p = p.read().await;
                    p.enqueue(fellow.clone()).await;
                    p.enqueue(channel_info.clone()).await;
                };
            }
        }
    };

    {
        let player_sessions = ctx.player_sessions.write().await;
        let player_sessions_map = player_sessions.token_map.read().await;
        if let Some(p) = player_sessions_map.get(&spectating_token) {
            p.read()
                .await
                .enqueue(packets::spectator_left(ctx.id))
                .await;
        };
        info!("{} is no longer watching {}.", ctx.name, spectating.name)
    }
}

#[inline(always)]
pub async fn spectate_frames<'a>(ctx: &HandlerContext<'a>) {
    let raw = PayloadReader::new(ctx.payload).read_raw().to_vec();

    let k = ctx.player_sessions.read().await;
    let p = k.id_session_map.read().await;
    let mut s = vec![];

    for i in ctx.data.spectators.iter() {
        s.push(i);
    }
    for i in s {
        if let Some(j) = p.get(i) {
            j.read().await.enqueue(raw.clone()).await;
        }
    }
    info!("sb");
}

pub async fn spectate_cant<'a>(ctx: &HandlerContext<'a>) {
    let data = packets::spectator_cant_spectate(ctx.id);
    let target = ctx.data.spectating.unwrap();

    let m = ctx.player_sessions.write().await;
    let z = m.id_session_map.read().await;
    let target = z.get(&target).unwrap().read().await;
    target.enqueue(data.clone()).await;

    for i in target.spectators.iter() {
        if let Some(player) = z.get(i) {
            player.read().await.enqueue(data.clone()).await;
        }
    }
    info!("sb");
}
