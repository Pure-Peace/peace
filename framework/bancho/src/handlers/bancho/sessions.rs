use ntex::web::types::Data;

use crate::objects::Bancho;

/// Auto PlayerSession recycle
#[inline]
pub async fn recycle_handler(bancho: &Data<Bancho>) {
    let session_timeout = read_lock!(bancho.config)
        .data
        .session_recycle
        .session_timeout;
    // Get deactive user token list
    let deactive_list = read_lock!(bancho.player_sessions)
        .deactive_token_list(session_timeout)
        .await;

    // If not any deactive sessions, just break
    if deactive_list.len() == 0 {
        return;
    };

    debug!("recycle_handler: session recycle task start!");
    let mut recycled_sessions_count = 0i32;
    let session_recycle_start = std::time::Instant::now();

    // Logout each deactive sessions
    for token in deactive_list {
        match write_lock!(bancho.player_sessions)
            .logout(&token, Some(&bancho.channel_list))
            .await
        {
            Some(player) => {
                recycled_sessions_count += 1;
                debug!(
                    "recycle_handler: deactive user {}({}) has been recycled.",
                    player.name, player.id
                )
            }
            None => {}
        }
    }

    // Done
    let session_recycle_end = session_recycle_start.elapsed();
    debug!(
        "recycle_handler: session recycle task complete in {:.2?}; recycled: {} sessions.",
        session_recycle_end, recycled_sessions_count
    );
}
