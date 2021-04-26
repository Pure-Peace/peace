use actix_web::{
    error, get, http::Method, web::BytesMut, web::Data, web::Path, web::Payload, Error,
    HttpRequest, HttpResponse,
};
use futures::StreamExt;
use num_traits::FromPrimitive;
use peace_constants::{api::UpdateUserTask, GameMode};
use peace_database::Database;
use serde::Deserialize;

use serde_json::json;

use crate::objects::{Bancho, Caches};

const PAYLOAD_MAX_SIZE: usize = 262_144;

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
pub struct ReCreateScoreTable {
    pub map_md5: String,
    pub mode: u8,
}

/// GET "/api/v1/recreate_score_table"
#[get("/recreate_score_table/{map_md5}/{mode}")]
pub async fn recreate_score_table(
    req: HttpRequest,
    data: Path<ReCreateScoreTable>,
    bancho: Data<Bancho>,
    database: Data<Database>,
    caches: Data<Caches>,
) -> Result<HttpResponse, Error> {
    if !peace_utils::web::header_checker(
        &req,
        "peace_key",
        &bancho.local_config.data.pp_server.peace_key,
    ) {
        return Err(error::ErrorUnauthorized("peace_key is invalid"));
    }

    let mode = match GameMode::from_u8(data.mode) {
        Some(m) => m,
        None => return Err(error::ErrorNotFound("mode is invalid")),
    };

    let _temp_table = Bancho::create_score_table(
        &data.map_md5,
        &mode.full_name(),
        mode.pp_is_best(),
        &database,
        &caches,
        true,
    )
    .await;

    return Ok(HttpResponse::Ok().body("ok"));
}

/// Update user(multiple or single)'s stats in game
///
/// GET "/api/v1/update_user_stats?player_id={player_id}&mode={game_mode_value}&recalc={true/false}"
///
/// POST "/api/v1/update_user_stats"
///
/// TODO: maybe we should broadcast stats packet to all users
pub async fn update_user_stats(
    req: HttpRequest,
    mut payload: Payload,
    bancho: Data<Bancho>,
    database: Data<Database>,
) -> Result<HttpResponse, Error> {
    if !peace_utils::web::header_checker(
        &req,
        "peace_key",
        &bancho.local_config.data.pp_server.peace_key,
    ) {
        return Err(error::ErrorUnauthorized("peace_key is invalid"));
    }

    let tasks = match req.method() {
        &Method::GET => {
            vec![serde_qs::from_str::<UpdateUserTask>(req.query_string())
                .map_err(|_| error::ErrorNotFound("param error"))?]
        }
        &Method::POST => {
            // payload is a stream of Bytes objects
            let mut body = BytesMut::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                // limit max size of in-memory payload
                if (body.len() + chunk.len()) > PAYLOAD_MAX_SIZE {
                    return Err(error::ErrorBadRequest("overflow"));
                }
                body.extend_from_slice(&chunk);
            }
            serde_json::from_slice::<Vec<UpdateUserTask>>(&body)?
        }
        _ => return Err(error::ErrorMethodNotAllowed("only allowed GET, POST")),
    };

    let mut success = 0i32;
    let mut failed = 0i32;
    let start = std::time::Instant::now();
    for UpdateUserTask {
        player_id,
        mode: mode_val,
        recalc,
    } in tasks.iter()
    {
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

        let pp_acc_result = if *recalc {
            peace_utils::peace::player_calculate_pp_acc(*player_id, &mode.full_name(), &database)
                .await
        } else {
            peace_utils::peace::player_get_pp_acc(*player_id, &mode, &database).await
        };

        // If player is online, we should update stats and send player_updates packet to them
        if let Some(result) = pp_acc_result {
            let mut p = p.write().await;
            // If player's current mode is this mode,
            // update current mode then clone into stats cache
            if p.game_status.mode == mode {
                p.stats.pp_v2 = result.pp;
                p.stats.accuracy = result.acc;
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
                    stats.pp_v2 = result.pp;
                    stats.accuracy = result.acc;
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

    Ok(HttpResponse::Ok().body(json!({
        "success": success,
        "failed": failed,
        "duration": end
    })))
}
