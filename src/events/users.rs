use super::depends::*;

use crate::{constants::PresenceFilter, packets};
use num_traits::FromPrimitive;

#[inline(always)]
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
// TODO: fix play_mods (mutiple mods support)
pub async fn change_action(
    payload: &[u8],
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let mut reader = PayloadReader::new(&payload);
    let (action, info, playing_beatmap_md5, play_mods, game_mode, playing_beatmap_id) = (
        reader.read_integer::<u8>().await,
        reader.read_string().await,
        reader.read_string().await,
        reader.read_integer::<u32>().await,
        reader.read_integer::<u8>().await,
        reader.read_integer::<i32>().await,
    );

    println!(
        "{} {} {} {} {} {}",
        action, info, playing_beatmap_md5, play_mods, game_mode, playing_beatmap_id
    );

    let player_sessions = player_sessions.read().await;
    match player_sessions
        .handle_player_get(token, move |p| {
            p.update_status(
                Action::from_u8(action)?,
                info,
                playing_beatmap_md5,
                playing_beatmap_id,
                PlayMods::from_u32(play_mods)?,
                GameMode::from_u8(game_mode)?,
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
            error!("Failed to update player's status! <OSU_CHANGE_ACTION>")
        }
    };
}

#[inline(always)]
pub async fn add_friend(
    payload: &[u8],
    database: &Database,
    player_data: &PlayerData,
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let target = PayloadReader::new(&payload).read_integer::<i32>().await;

    if !player_sessions.read().await.id_is_exists(&target).await {
        info!(
            "Player {}({}) tries to add a offline {} to friends.",
            player_data.name, player_data.id, target
        );
        return;
    };

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
pub async fn remove_friend(
    payload: &[u8],
    database: &Database,
    player_data: &PlayerData,
    token: &String,
    player_sessions: &Data<RwLock<PlayerSessions>>,
) {
    let target = PayloadReader::new(&payload).read_integer::<i32>().await;

    if !player_sessions.read().await.id_is_exists(&target).await {
        info!(
            "Player {}({}) tries to remove a offline {} from friends.",
            player_data.name, player_data.id, target
        );
        return;
    };

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
