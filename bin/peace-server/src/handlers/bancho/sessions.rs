use ntex::web::types::Data;

use crate::objects::Bancho;

/// Auto PlayerSession recycle
#[inline(always)]
pub async fn recycle_handler(bancho: &Data<Bancho>) {
    let session_timeout = bancho
        .config
        .read()
        .await
        .data
        .session_recycle
        .session_timeout;
    // Get deactive user token list
    let deactive_list = bancho
        .player_sessions
        .read()
        .await
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
        match bancho
            .player_sessions
            .write()
            .await
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
