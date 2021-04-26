use actix_web::{get, post, web::Data, web::Json, HttpResponse};
use num_traits::FromPrimitive;
use peace_constants::GameMode;
use peace_database::Database;
use serde::Deserialize;
use serde_json::json;

use crate::objects::Bancho;

/// GET "/api/v1"
#[get("")]
pub async fn index() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/is_online"
#[get("/is_online")]
pub async fn is_online() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/online_users"
#[get("/online_users")]
pub async fn online_users() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/server_status"
#[get("/server_status")]
pub async fn server_status() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/verified_status"
#[get("/verified_status")]
pub async fn verified_status() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/ci_trigger"
#[get("/ci_trigger")]
pub async fn ci_trigger() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

/// GET "/api/v1/bot_message"
#[get("/bot_message")]
pub async fn bot_message() -> HttpResponse {
    let contents = "Hello api v1!";
    HttpResponse::Ok().body(contents)
}

#[derive(Deserialize)]
pub struct UpdateUserTasks {
    // player_id, mode_val
    pub player_and_mode: Vec<(i32, u8)>,
    pub peace_key: String,
}

/// POST "/api/v1/update_user_stats"
/// TODO: maybe we should broadcast stats packet to all users
#[post("/update_user_stats")]
pub async fn update_user_stats(
    data: Json<UpdateUserTasks>,
    bancho: Data<Bancho>,
    database: Data<Database>,
) -> HttpResponse {
    if data.peace_key != bancho.local_config.data.pp_server.peace_key {
        return HttpResponse::NonAuthoritativeInformation().body("0");
    }
    let mut success = 0;
    let mut failed = 0;
    let start = std::time::Instant::now();
    for (player_id, mode_val) in data.player_and_mode.iter() {
        let mode = match GameMode::from_u8(*mode_val) {
            Some(m) => m,
            None => {
                failed += 1;
                warn!(
                    "[update_user_stats] Invalid mode_val: {}, player_id: {}",
                    mode_val, player_id
                );
                continue;
            }
        };

        let p = match bancho
            .player_sessions
            .read()
            .await
            .get_player_by_id(*player_id)
            .await
        {
            Some(p) => p,
            None => {
                success += 1;
                continue;
            }
        };

        // If player is online, we should update stats and send player_updates packet to them
        if let Some((pp, acc)) =
            peace_utils::peace::player_get_pp_acc(*player_id, &mode, &database).await
        {
            let mut p = p.write().await;
            // If player's current mode is this mode,
            // update current mode then clone into stats cache
            if p.game_status.mode == mode {
                p.stats.pp_v2 = pp;
                p.stats.accuracy = acc;
                p.stats.calc_rank_from_database(&mode, &database).await;
                p.stats.update_time();
                // Send stats packet
                p.enqueue(p.stats_packet()).await;
                // Cache stats
                let s = p.stats.clone();
                p.stats_cache.insert(mode.clone(), s);
            } else {
                // Otherwise, just update stats cache
                if let Some(mut stats) = p.stats_cache.get_mut(&mode) {
                    stats.pp_v2 = pp;
                    stats.accuracy = acc;
                    stats.calc_rank_from_database(&mode, &database).await;
                    stats.update_time();
                }
            }
            success += 1;
            debug!(
                "[update_user_stats] Player {}({})'s {:?} stats now updated.",
                p.name, p.id, mode
            );
        }
    }
    let end = start.elapsed();
    info!(
        "[update_user_stats] Success update {} user's stats, time spent: {:?}",
        success, end
    );
    HttpResponse::Ok().body(json!({
        "success": success,
        "failed": failed,
        "duration": end
    }))
}
