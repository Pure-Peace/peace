use super::depends::*;
use crate::handlers::web::get;
use crate::utils;

const BASE: &'static str = "Bancho /web [GET]";

pub async fn handler(
    req: HttpRequest,
    path: Path<String>,
    counter: Data<IntCounterVec>,
    player_sessions: Data<RwLock<PlayerSessions>>,
    database: Data<Database>,
    bancho_config: Data<RwLock<BanchoConfig>>,
    argon2_cache: Data<RwLock<Argon2Cache>>,
    geo_db: Data<Option<Reader<Mmap>>>,
) -> HttpResponse {
    counter.with_label_values(&["/web", "get", "start"]).inc();
    // Get real request ip
    let request_ip = match utils::get_realip(&req).await {
        Ok(ip) => ip,
        Err(_) => {
            return HttpResponse::BadRequest().body("bad requests");
        }
    };

    let ctx = || Context {
        req: &req,
        counter: &counter,
        player_sessions: &player_sessions,
        database: &database,
        bancho_config: &bancho_config,
        argon2_cache: &argon2_cache,
        geo_db: &geo_db,
    };

    debug!("{} Path: <{}>; ip: {}", BASE, path, request_ip);

    let handle_start = std::time::Instant::now();
    let handle_path = path.replace(".php", "");
    let resp = match handle_path.as_str() {
        "lastfm" => get::lastfm(&ctx()).await,
        "osu-rate" => get::osu_rate(&ctx()).await,
        "osu-addfavourite" => get::osu_add_favourite(&ctx()).await,
        /*"osu-markasread" => {}*/
        "osu-getreplay" => get::osu_get_replay(&ctx()).await,
        "osu-getfavourites" => get::osu_get_favourites(&ctx()).await,
        "osu-getfriends" => get::osu_get_friends(&ctx()).await,
        "osu-getseasonal" => get::osu_get_seasonal(&ctx()).await,
        /* "osu-get-beatmap-topic" => {}
        "osu-search" => {}
        "osu-search-set" => {}  */
        "osu-osz2-getscores" => get::osu_osz2_get_scores(&ctx()).await,
        //"osu-osz2-bmsubmit-getid" => {}
        _ => {
            warn!("{} Unimplemented path: <{}>", BASE, path);
            HttpResponse::Ok().body("ok")
        }
    };

    let handle_end = handle_start.elapsed();
    info!(
        "{} Path: <{}> done; time spent: {:?}",
        BASE, path, handle_end
    );

    resp
}
