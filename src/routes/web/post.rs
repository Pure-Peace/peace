use super::depends::*;
use crate::utils;
use crate::{
    handlers::web::post,
    objects::{Bancho, Caches},
};

const BASE: &'static str = "Bancho /web [POST]";

pub async fn handler(
    req: HttpRequest,
    path: Path<String>,
    bancho: Data<Bancho>,
    counter: Data<IntCounterVec>,
    database: Data<Database>,
    geo_db: Data<Option<Reader<Mmap>>>,
    global_cache: Data<Caches>,
    payload: Multipart,
) -> HttpResponse {
    counter.with_label_values(&["/web", "post", "start"]).inc();
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
        player_sessions: &bancho.player_sessions,
        database: &database,
        bancho_config: &bancho.config,
        geo_db: &geo_db,
        global_cache: &global_cache,
        osu_api: &bancho.osu_api,
    };

    debug!("{} Path: <{}>; ip: {}", BASE, path, request_ip);

    let handle_start = std::time::Instant::now();
    let handle_path = path.replace(".php", "");
    let resp = match handle_path.as_str() {
        /* "osu-session" => {} */
        "osu-error" => post::osu_error(&ctx(), payload).await,
        /* "osu-get-beatmapinfo" => {}
        "osu-submit-modular-selector" => {}
        "osu-comment" => {}
        "osu-screenshot" => {}
        "osu-osz2-bmsubmit-post" => {}
        "osu-osz2-bmsubmit-upload" => {} */
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
