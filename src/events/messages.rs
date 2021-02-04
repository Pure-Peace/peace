use crate::packets;

use super::depends::*;

#[inline(always)]
pub async fn public<'a>(ctx: &HandlerContext<'a>) {
    // TODO: check player is slienced?

    let mut payload = PayloadReader::new(ctx.payload);
    let message = payload.read_message().await;

    // Check channel
    let channel_list = ctx.channel_list.read().await;
    match channel_list.get(&message.target) {
        Some(channel) => {
            // TODO: check player's priv?

            // TODO: Limit the length of message content?
            // Send message done
            channel
                .broadcast(
                    ctx.player_sessions,
                    ctx.name,
                    ctx.id,
                    &message.content,
                    false,
                )
                .await;

            // Drop locks
            drop(channel_list);

            info!(
                "{}({}) <pub> @ {}: {}",
                ctx.name, ctx.id, message.target, message.content
            );
        }
        None => {
            warn!(
                "Player {}({}) try send message to non-existent channel: {}",
                ctx.name, ctx.id, message.target
            );
        }
    }
}

#[inline(always)]
pub async fn private<'a>(ctx: &HandlerContext<'a>) {
    // TODO: check player is slienced?

    let mut payload = PayloadReader::new(ctx.payload);
    let message = payload.read_message().await;

    // BanchoBot? current not exists
    if message.target == "BanchoBot" {
        return;
    }

    let player_sessions = ctx.player_sessions.read().await;
    let map = player_sessions.map.read().await;

    // Find target player
    match map.values().find(|target| target.name == message.target) {
        Some(target) => {
            // Active player (sender)
            let player = match map.get(ctx.token) {
                Some(player) => player,
                None => {
                    warn!(
                        "Failed to send private messages, player {}({}) has logout!",
                        ctx.name, ctx.id
                    );
                    return;
                }
            };

            // Target player only allowed friend's pm
            // Admin are not limited
            if (player.bancho_privileges & Privileges::Admin as i32) == 0
                && target.only_friend_pm_allowed
                && !target.friends.contains(&player.id)
            {
                // Blocked
                player.enqueue(packets::user_dm_blocked(&target.name)).await;
                warn!(
                    "Player {}({}) try send message to blocked-non-friends player: {}({})",
                    &player.name, player.id, target.name, target.id
                );
                return;
            }

            // TODO: target is slienced
            if false {
                player.enqueue(packets::target_silenced(&target.name)).await;
            }

            // TODO: if target is bot, handle it --

            // TODO: Limit the length of message content?
            // Send message done
            target
                .enqueue(
                    packets::send_message(
                        &player.name,
                        player.id,
                        &message.content,
                        &message.target,
                    )
                    .await,
                )
                .await;

            // Drop locks
            drop(map);
            drop(player_sessions);

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
