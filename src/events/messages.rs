use crate::packets;

use super::depends::*;

#[inline(always)]
/// #1: OSU_SEND_PUBLIC_MESSAGE
pub async fn public<'a>(ctx: &HandlerContext<'a>) {
    // TODO: check player is slienced?

    let mut payload = PayloadReader::new(ctx.payload);
    let mut message = payload.read_message().await;

    let channel_name = match message.target.as_str() {
        "#spectator" => {
            if ctx.data.spectating.is_some() {
                format!("#spec_{}", ctx.data.spectating.unwrap())
            } else if ctx.data.spectators.len() > 0 {
                format!("#spec_{}", ctx.id)
            } else {
                return;
            }
        }
        "#multiplayer" => {
            // TODO: multiplayer chat
            String::new()
        }
        x => x.to_string(),
    };

    let cfg_r = ctx.bancho.config.read().await;
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

    // Check channel
    let channel_list = ctx.bancho.channel_list.read().await;
    match channel_list.get(&channel_name) {
        Some(channel) => {
            // TODO: check player's priv?

            // Send message done
            channel
                .broadcast(ctx.name, ctx.u_name, ctx.id, &message.content, false)
                .await;

            // Drop locks
            drop(channel_list);

            info!(
                "{}({}) <pub> @ {}: {}",
                ctx.name, ctx.id, channel_name, message.content
            );
        }
        None => {
            warn!(
                "Player {}({}) try send message to non-existent channel: {}",
                ctx.name, ctx.id, channel_name
            );
        }
    }
}

#[inline(always)]
/// #24: OSU_SEND_PRIVATE_MESSAGE
pub async fn private<'a>(ctx: &HandlerContext<'a>) {
    // TODO: check player is slienced?

    let mut payload = PayloadReader::new(ctx.payload);
    let mut message = payload.read_message().await;

    // BanchoBot? current not exists
    if message.target == "BanchoBot" {
        return;
    }

    let cfg_r = ctx.bancho.config.read().await;
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

    let player_sessions = ctx.bancho.player_sessions.read().await;
    let name_session_map = player_sessions.name_session_map.read().await;

    // Find target player
    match name_session_map.get(&message.target) {
        Some(target) => {
            // Active player (sender)
            let target = target.read().await;
            let player = ctx.weak_player.upgrade();

            if player.is_none() {
                warn!(
                    "Failed to send private messages, player {}({}) has logout!",
                    ctx.name, ctx.id
                );
                return;
            };
            let player = player.as_ref().unwrap().read().await;

            let target_name = if player.settings.display_u_name {
                target.try_u_name()
            } else {
                target.name.clone()
            };

            // Target player only allowed friend's pm
            // Admin are not limited
            if (player.bancho_privileges & Privileges::Admin as i32) == 0
                && target.only_friend_pm_allowed
                && !target.friends.contains(&player.id)
            {
                // Blocked
                player.enqueue(packets::user_dm_blocked(&target_name)).await;
                warn!(
                    "Player {}({}) try send message to blocked-non-friends player: {}({})",
                    &player.name, player.id, target.name, target.id
                );
                return;
            }

            // TODO: target is slienced
            if false {
                player.enqueue(packets::target_silenced(&target_name)).await;
            }

            // TODO: if target is bot, handle it --

            // TODO: Limit the length of message content?
            // Send message done
            target
                .enqueue(
                    packets::send_message(
                        &if target.settings.display_u_name {
                            player.try_u_name()
                        } else {
                            player.name.clone()
                        },
                        player.id,
                        &message.content,
                        &message.target,
                    )
                    .await,
                )
                .await;

            info!(
                "{}({}) <pvt> @ {}: {}",
                ctx.name, ctx.id, message.target, message.content
            );
        }
        // Find None
        _ => {
            warn!(
                "Player {}({}) try send message to non-existent (or offline) player: {}",
                ctx.name, ctx.id, message.target
            );
        }
    }
}
