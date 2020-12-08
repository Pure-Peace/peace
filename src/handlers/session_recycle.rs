use actix_web::web::Data;
use async_std::sync::RwLock;

use crate::objects::PlayerSessions;

/// Auto PlayerSession recycle
#[inline(always)]
pub async fn session_recycle_handler(
    player_sessions: &Data<RwLock<PlayerSessions>>,
    session_timeout: i64,
) {
    // Get deactive user token list
    let deactive_list = player_sessions
        .read()
        .await
        .deactive_token_list(session_timeout)
        .await;

    // If not any deactive sessions, just break
    if deactive_list.len() == 0 {
        return;
    };

    info!("session recycle task start!");
    let mut recycled_sessions_count = 0;
    let session_recycle_start = std::time::Instant::now();

    // Logout each deactive sessions
    for token in deactive_list {
        match player_sessions.write().await.logout(&token).await {
            Some((_token, player)) => {
                recycled_sessions_count += 1;
                warn!(
                    "deactive user {}({}) has been recycled.",
                    player.name, player.id
                )
            }
            None => {}
        }
    }

    // Done
    let session_recycle_end = session_recycle_start.elapsed();
    info!(
        "session recycle task complete in {:.2?}; recycled: {} sessions.",
        session_recycle_end, recycled_sessions_count
    );
}
