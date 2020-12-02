use actix_web::web::Data;
use async_std::sync::RwLock;
use chrono::Local;

use crate::objects::{PlayerData, PlayerSessions};

/// Auto PlayerSession recycle
#[inline(always)]
pub async fn session_recycle_handler(
    player_sessions: &Data<RwLock<PlayerSessions>>,
    session_timeout: i64,
) {
    let mut recycled_sessions_count = 0;
    let session_recycle_start = std::time::Instant::now();

    // Get read lock
    let sessions = player_sessions.read().await;
    let session_map = sessions.map.read().await;

    // If not any sessions, just break
    if session_map.len() == 0 {
        return ();
    };

    debug!("session recycle task start!");
    let map_data: Vec<(String, PlayerData)> = session_map
        .iter()
        .map(|(token, player)| (token.to_string(), PlayerData::from(player)))
        .collect();
    // Drop lock before handled
    drop(session_map);
    for (token, player) in map_data {
        if Local::now().timestamp() - player.last_active_time.timestamp() > session_timeout {
            match sessions.logout(token).await {
                Some((_token, player)) => {
                    recycled_sessions_count += 1;
                    warn!(
                        "deactive user {}({}) has been recycled.",
                        player.name, player.id
                    )
                },
                None => {}
            }
        }
    }
    let session_recycle_end = session_recycle_start.elapsed();
    debug!("session recycle task complete in {:.2?}; recycled: {} sessions.", session_recycle_end, recycled_sessions_count);
}
