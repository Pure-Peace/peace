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
    match player_sessions
        .write()
        .await
        .handle_player(token, move |p| {
            p.status.action = Action::from_u8(action)?;
            p.status.info = info;
            p.status.playing_beatmap_md5 = playing_beatmap_md5;
            p.status.play_mods = PlayMods::from_u32(play_mods)?;
            p.status.game_mode = GameMode::from_u8(game_mode)?;
            p.status.playing_beatmap_id = playing_beatmap_id;
            Some(())
        })
        .await
    {
        Ok(()) => {}
        Err(()) => {
            error!("Failed to update player's status! <OSU_CHANGE_ACTION>")
        }
    };
}
