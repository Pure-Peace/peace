use super::depends::*;
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
    caches: Data<Caches>,
    payload: Multipart,
) -> HttpResponse {
    counter.with_label_values(&["/web", "post", "start"]).inc();
    // Get real request ip
    let request_ip = match peace_utils::web::get_realip(&req).await {
        Ok(ip) => ip,
        Err(_) => {
            return HttpResponse::BadRequest().body("bad requests");
        }
    };

    let ctx = || Context {
        req,
        counter: &counter,
        bancho: &bancho,
        database: &database,
        geo_db: &geo_db,
        caches: &caches,
    };

    debug!("{} Path: <{}>; ip: {}", BASE, path, request_ip);

    let handle_start = std::time::Instant::now();
    let handle_path = path.replace(".php", "");
    let resp = match handle_path.as_str() {
        /* "osu-session" => {} */
        "osu-error" => post::osu_error(&ctx(), payload).await,
        /*  "osu-get-beatmapinfo" => {} */
        "osu-submit-modular-selector" => post::osu_submit_modular(&ctx(), payload).await,
        /*"osu-comment" => {}
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
