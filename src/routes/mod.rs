use actix_web::web::{get, post, scope, ServiceConfig};
use actix_web::{dev::HttpServiceFactory, guard};

use crate::settings::model::Settings;

pub mod api;
pub mod bancho;
pub mod debug;
pub mod default;
pub mod geoip;
pub mod web;

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig, settings: Settings) {
    init_default(cfg);
    cfg.service(init_bancho());
    cfg.service(init_web());
    cfg.service(init_api_v1());
    cfg.service(init_api_v2());

    if settings.geoip.web_api {
        cfg.service(init_geoip());
    };

    // !warning: only debug!
    if settings.debug == true {
        init_debug(cfg)
    }
}

// Init geoip api
fn init_geoip() -> impl HttpServiceFactory {
    use geoip::*;
    let path = "/geoip";
    scope(path).service(index).service(geo_ip)
}

fn init_debug(cfg: &mut ServiceConfig) {
    use debug::*;
    cfg.service(test_pg);
    cfg.service(test_redis);
    cfg.service(test_player_read);
    cfg.service(test_async_lock);
    cfg.service(test_player_money_add);
    cfg.service(test_player_money_reduce);
    cfg.service(test_player_money_reduce_special);
    cfg.service(pleyer_sessions_all);
    cfg.service(pleyer_maps_info);
    cfg.service(player_channels_all);
    cfg.service(pleyer_sessions_kick);
    cfg.service(pleyer_sessions_kick_uid);
    cfg.service(test_geo_ip);
}

/// Routes for default
fn init_default(cfg: &mut ServiceConfig) {
    use default::*;
    cfg.service(index);
}

/// Routes for bancho
fn init_bancho() -> impl HttpServiceFactory {
    scope("/bancho")
        .route("/", get().to(bancho::get::handler))
        .route(
            "/",
            post()
                .guard(guard::Header("user-agent", "osu!"))
                .to(bancho::post::handler),
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
    use api::v1::*;
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
    use api::v2::*;
    scope("/api/v2").service(index)
}
