#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Data, Query};
use actix_web::{get, HttpRequest, HttpResponse};
use async_std::sync::RwLock;
use prometheus::IntCounterVec;

use crate::{settings::bancho::BanchoConfig, utils};

use super::data::*;

#[inline(always)]
fn unimplemented(route: &str) -> HttpResponse {
    warn!("[GET] Unimplemented route request: {}", route);
    HttpResponse::Ok().body("ok")
}

#[get("/check-updates.php")]
pub async fn check_updates(
    req: HttpRequest,
    Query(query): Query<CheckUpdates>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    unimplemented("check-updates.php")
}

#[get("/bancho_connect.php")]
pub async fn bancho_connect(req: HttpRequest, Query(query): Query<BanchoConnect>) -> HttpResponse {
    unimplemented("bancho_connect.php")
}

#[get("/lastfm.php")]
pub async fn lastfm(
    req: HttpRequest,
    Query(query): Query<Lastfm>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    unimplemented("lastfm.php")
}

#[get("/osu-rate.php")]
pub async fn osu_rate(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-rate.php")
}

#[get("/osu-addfavourite.php")]
pub async fn osu_add_favourite(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-addfavourite.php")
}

#[get("/osu-markasread.php")]
pub async fn osu_mark_as_read(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-markasread.php")
}

#[get("/osu-getreplay.php")]
pub async fn osu_get_replay(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getreplay.php")
}
#[get("/osu-getfavourites.php")]
pub async fn osu_get_favourites(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getfavourites.php")
}

#[get("/osu-getfriends.php")]
pub async fn osu_get_friends(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-getfriends.php")
}

#[get("/osu-getseasonal.php")]
/// Seasonal background images
/// 
/// Locate on database -> bancho.config.seasonal_backgrounds
/// String Array
pub async fn osu_get_seasonal(bancho_config: Data<RwLock<BanchoConfig>>) -> HttpResponse {
    if let Some(background_images) = &bancho_config.read().await.seasonal_backgrounds {
        return HttpResponse::Ok().json(background_images);
    };

    HttpResponse::Ok().body("[]")
}

#[get("osu-get-beatmap-topic.php")]
pub async fn osu_get_beatmap_topic(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-get-beatmap-topic.php")
}

#[get("/osu-search.php")]
pub async fn osu_search(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-search.php")
}

#[get("/osu-search-set.php")]
pub async fn osu_search_set(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-search-set.php")
}

#[get("/osu-osz2-getscores.php")]
pub async fn osu_osz2_get_scores(req: HttpRequest) -> HttpResponse {
    // unimplemented("osu-osz2-getscores.php")
    HttpResponse::Ok().body("-1|false")
}

#[get("osu-osz2-bmsubmit-getid")]
pub async fn osu_osz2_bm_submit_getid(req: HttpRequest) -> HttpResponse {
    unimplemented("osu-osz2-bmsubmit-getid")
}
