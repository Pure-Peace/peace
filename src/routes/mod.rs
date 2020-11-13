#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::{middleware, web::{get, post, scope, ServiceConfig}};
use actix_web::{dev::HttpServiceFactory, guard};

pub mod web;
pub mod api_v1;
pub mod api_v2;
pub mod bancho;
pub mod root;

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig) {
    init_root(cfg);
    cfg.service(init_bancho());
    cfg.service(init_web());
    cfg.service(init_api_v1());
    cfg.service(init_api_v2());
}

/// Routes for root
fn init_root(cfg: &mut ServiceConfig) {
    use root::*;
    cfg.service(index);
}

/// Routes for bancho
fn init_bancho() -> impl HttpServiceFactory {
    scope("/bancho")
        .route("", get().to(bancho::get))
        .route("", post().guard(guard::Header("user-agent", "osu!")).to(bancho::post),
    )
}

/// Routes for web
fn init_web() -> impl HttpServiceFactory {
    use web::*;
    scope("/web")
        .service(lastfm)
        .service(check_updates)
        .service(osu_session)
        .service(osu_error)
        .service(bancho_connect)
}

/// Routes for api_v1
fn init_api_v1() -> impl HttpServiceFactory {
    use api_v1::*;
    scope("/api/v1")
        .service(index)
        .service(is_online)
        .service(online_users)
        .service(server_status)
        .service(verified_status)
        .service(ci_trigger)
        .service(bot_message)
}

/// Routes for api_v2
fn init_api_v2() -> impl HttpServiceFactory {
    use api_v2::*;
    scope("/api/v2").service(index)
}
