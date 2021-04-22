use std::sync::{atomic::Ordering, Weak};

use async_std::sync::RwLockReadGuard;

use crate::{
    objects::{Channel, Player},
    packets,
};

use super::depends::*;

#[inline(always)]
pub async fn try_remove_spectator<'a>(
    player_id: i32,
    player_name: &String,
    weak_player: &Weak<RwLock<Player>>,
    spectating_id: i32,
    channel_list: &Data<RwLock<ChannelList>>,
    player_sessions: &RwLockReadGuard<'_, PlayerSessions>,
) {
    // First, Remove spectating status from me
    if let Some(player) = weak_player.upgrade() {
        player.write().await.spectating = None;
    }

    // And, remove me from spectating player
    let id_session_map = player_sessions.id_session_map.read().await;
    let non_spectators = if let Some(spectating_target) = id_session_map.get(&spectating_id) {
        let mut t = spectating_target.write().await;
        t.spectators.remove(&player_id);
        t.spectators.len() == 0
    } else {
        false
    };

    let spectating_channel_name = format!("#spec_{}", spectating_id);
    {
        let mut channel_list = channel_list.write().await;
        if let Some(spectating_channel) = channel_list.get_mut(&spectating_channel_name) {
            // Remove me from spectating channel
            spectating_channel
                .leave(player_id, Some(&*player_sessions))
                .await;

            // The spectating player have not spectators
            if non_spectators {
                // Remove spectating player from spectating channel
                spectating_channel
                    .leave(spectating_id, Some(&*player_sessions))
                    .await;
            } else {
                let fellow_data = packets::fellow_spectator_left(player_id);
                let channel_info = spectating_channel.channel_info_packet();

                if let Some(spectating_target) = id_session_map.get(&spectating_id) {
                    let t = spectating_target.write().await;
                    // Send channel info to spectating player
                    t.enqueue(channel_info.clone()).await;

                    // Send data to each spectators
                    for id in t.spectators.iter() {
                        if let Some(player) = id_session_map.get(&id) {
                            let p = player.read().await;
                            p.enqueue(fellow_data.clone()).await;
                            p.enqueue(channel_info.clone()).await;
                        }
                    }
                }
            }

            // If spectating channel is empty, remove it
            if spectating_channel.player_count.load(Ordering::SeqCst) == 0 {
                drop(spectating_channel);
                channel_list.remove(&spectating_channel_name);
            };
        }
    }

    if let Some(spectating_target) = id_session_map.get(&spectating_id) {
        let t = spectating_target.read().await;
        t.enqueue(packets::spectator_left(player_id)).await;
        debug!(
            "Player {}({}) is no longer watching {}({}).",
            t.name, t.id, player_name, player_id
        )
    }
}

#[inline(always)]
pub async fn create_specate_channel_if_not_exists(
    player_id: i32,
    player_name: String,
    channel_list: &Data<RwLock<ChannelList>>,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) -> String {
    let channel_name = format!("#spec_{}", player_id);

    if !channel_list.read().await.contains_key(&channel_name) {
        let channel = Channel::new(
            channel_name.clone(),
            format!("{}({}) 's spectator channel!", player_name, player_id),
            1,
            1,
            false,
            true,
            player_sessions.clone().into_inner(),
        );

        channel.join(player_id, None).await;

        channel_list
            .write()
            .await
            .insert(channel_name.clone(), channel);

        debug!("Spectate channel {} created.", channel_name);
    };

    channel_name
}

#[inline(always)]
/// #16: OSU_SPECTATE_START
///
pub async fn spectate_start<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let target_id = PayloadReader::new(ctx.payload)
        .read_integer::<i32>()
        .await?;

    // -1 is BanchoBot, not exists
    if target_id == -1 {
        return None;
    }

    let player_sessions = ctx.bancho.player_sessions.read().await;

    // Specate an offline player is not allowed!
    let target_name = match player_sessions.id_session_map.read().await.get(&target_id) {
        Some(target) => target.read().await.name.clone(),
        None => {
            warn!(
                "Player {}({}) tries to spectate an offline user {}.",
                ctx.name, ctx.id, target_id
            );
            return None;
        }
    };

    // If already spectating
    if ctx.data.spectating.is_some() {
        try_remove_spectator(
            ctx.id,
            &ctx.name,
            ctx.weak_player,
            ctx.data.spectating.unwrap(),
            &ctx.bancho.channel_list,
            &player_sessions,
        )
        .await;
    }

    // Create channel
    let channel_name = create_specate_channel_if_not_exists(
        target_id,
        target_name,
        &ctx.bancho.channel_list,
        &ctx.bancho.player_sessions,
    )
    .await;

    // Try join channel
    {
        let channel_list = ctx.bancho.channel_list.read().await;
        let channel = channel_list.get(&channel_name);
        if channel.is_none() {
            warn!("Failed to create spectate channel {}.", channel_name);
            return None;
        }
        if !channel.unwrap().join(ctx.id, Some(&*player_sessions)).await {
            warn!(
                "Player {}({}) failed to join spectate channel {}.",
                ctx.name, ctx.id, channel_name
            );
            return None;
        }
    }

    // Ready to send packet
    {
        let i_was_joined = packets::fellow_spectator_joined(ctx.id);
        let i_was_joined2 = packets::spectator_joined(ctx.id);

        let id_session_map = player_sessions.id_session_map.read().await;
        let (target, player) = (id_session_map.get(&target_id), ctx.weak_player.upgrade());
        if target.is_none() || player.is_none() {
            return None;
        }

        let mut target = target.unwrap().write().await;
        let mut player = player.as_ref().unwrap().write().await;

        for spectator_id in target.spectators.iter() {
            if let Some(spectator) = id_session_map.get(&spectator_id) {
                let s = spectator.read().await;
                s.enqueue(i_was_joined.clone()).await;
                player.enqueue(packets::fellow_spectator_joined(s.id)).await;
            }
        }
        target.spectators.insert(ctx.id);
        player.spectating = Some(target_id);

        target.enqueue(i_was_joined2).await;
        debug!(
            "Player {}({}) is specating {}({}) now.",
            ctx.name, ctx.id, target.name, target.id
        );
    };
    Some(())
}

#[inline(always)]
/// #17: OSU_SPECTATE_STOP (non-payload)
///
pub async fn spectate_stop<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let player_sessions = ctx.bancho.player_sessions.read().await;

    try_remove_spectator(
        ctx.id,
        &ctx.name,
        ctx.weak_player,
        ctx.data.spectating?,
        &ctx.bancho.channel_list,
        &player_sessions,
    )
    .await;
    Some(())
}

#[inline(always)]
/// #18: OSU_SPECTATE_FRAMES
///
pub async fn spectate_frames_received<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let data = packets::spectator_frames(ctx.payload.to_vec());

    let player_sessions = ctx.bancho.player_sessions.read().await;
    let id_session_map = player_sessions.id_session_map.read().await;

    // Send the spectate frames to our ctx's spectators
    for spectator_id in &ctx.data.spectators {
        if let Some(spectator) = id_session_map.get(spectator_id) {
            spectator.read().await.enqueue(data.clone()).await;
        }
    }
    Some(())
}

#[inline(always)]
/// #21: OSU_SPECTATE_CANT (non-payload)
///
pub async fn spectate_cant<'a>(ctx: &HandlerContext<'a>) -> Option<()> {
    let data = packets::spectator_cant_spectate(ctx.id);
    let spectate_target_id = ctx.data.spectating?;

    let player_sessions = ctx.bancho.player_sessions.read().await;
    let id_session_map = player_sessions.id_session_map.read().await;

    // Send packet
    if let Some(spectate_target) = id_session_map.get(&spectate_target_id) {
        let spectate_target = spectate_target.read().await;

        spectate_target.enqueue(data.clone()).await;

        for id in spectate_target.spectators.iter() {
            if let Some(spectator) = id_session_map.get(id) {
                spectator.read().await.enqueue(data.clone()).await;
            }
        }
    };
    Some(())
}
