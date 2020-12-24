use crate::packets;

use super::depends::*;

#[inline(always)]
pub async fn public(
    payload: &[u8],
    channel_list: &Data<RwLock<ChannelList>>,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    player_data: &PlayerData,
) {
    // TODO: check player is slienced?

    let mut payload = PayloadReader::new(payload);
    let message = payload.read_message().await;

    // Check channel
    let channel_list = channel_list.read().await;
    match channel_list.get(&message.target) {
        Some(channel) => {
            // TODO: check player's priv?

            // TODO: Limit the length of message content?
            // Send message done
            channel
                .broadcast(
                    &player_sessions,
                    &player_data.name,
                    player_data.id,
                    &message.content,
                    false,
                )
                .await;

            // Drop locks
            drop(channel_list);

            info!(
                "{}({}) <pub> @ {}: {}",
                &player_data.name, player_data.id, message.target, message.content
            );
        }
        None => {
            warn!(
                "Player {}({}) try send message to non-existent channel: {}",
                &player_data.name, player_data.id, message.target
            );
        }
    }
}

#[inline(always)]
pub async fn private(
    payload: &[u8],
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
    player_data: &PlayerData,
) {
    // TODO: check player is slienced?

    let mut payload = PayloadReader::new(payload);
    let message = payload.read_message().await;

    // BanchoBot? current not exists
    if message.target == "BanchoBot" {
        return;
    }

    let player_sessions = player_sessions.read().await;
    let map = player_sessions.map.read().await;

    // Find target player
    match map.values().find(|target| target.name == message.target) {
        Some(target) => {
            // Active player (sender)
            let player = match map.get(token) {
                Some(player) => player,
                None => {
                    warn!(
                        "Failed to send private messages, player {}({}) has logout!",
                        player_data.name, player_data.id
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
                &player_data.name, player_data.id, message.target, message.content
            );
        }
        // Find None
        _ => {
            warn!(
                "Player {}({}) try send message to non-existent player: {}",
                &player_data.name, player_data.id, message.target
            );
        }
    }
}
