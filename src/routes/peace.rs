use actix_web::web::{get, post, scope, ServiceConfig};
use actix_web::{dev::HttpServiceFactory, guard};

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig, config_data: &peace_settings::local::LocalConfigData) {
    init_default(cfg);
    cfg.service(init_bancho());
    cfg.service(init_web());
    cfg.service(init_api_v1());
    cfg.service(init_api_v2());

    if config_data.geoip.web_api {
        cfg.service(init_geoip());
    };

    // !warning: only debug!
    if config_data.debug == true {
        init_debug(cfg)
    }
}

// Init geoip api
fn init_geoip() -> impl HttpServiceFactory {
    use super::geoip::*;
    let path = "/geoip";
    scope(path).service(index).service(geo_ip)
}

fn init_debug(cfg: &mut ServiceConfig) {
    use super::debug::*;
    cfg.service(test_pg);
    cfg.service(test_redis);
    cfg.service(test_player_read);
    cfg.service(test_async_lock);
    cfg.service(test_player_money_add);
    cfg.service(test_player_money_reduce);
    cfg.service(test_player_money_reduce_special);
    cfg.service(player_sessions_all);
    cfg.service(player_maps_info);
    cfg.service(player_channels_all);
    cfg.service(player_sessions_kick);
    cfg.service(player_sessions_kick_uid);
    cfg.service(test_geo_ip);
    cfg.service(bancho_config_update);
    cfg.service(bancho_config_get);
    cfg.service(osu_api_test);
    cfg.service(osu_api_reload);
    cfg.service(osu_api_all);
    cfg.service(server_stop);
}

/// Routes for default
fn init_default(cfg: &mut ServiceConfig) {
    use super::default::*;
    cfg.service(index);
}

/// Routes for bancho
fn init_bancho() -> impl HttpServiceFactory {
    use super::bancho;
    scope("/bancho")
        .route("/", get().to(bancho::get::handler))
        .route(
            "/",
            post()
                .guard(guard::Header("user-agent", "osu!"))
                .to(bancho::post::handler),
        )
        .service(bancho::osu_register)
}

/// Routes for web
fn init_web() -> impl HttpServiceFactory {
    use super::web;
    scope("/web")
        .route("/{path}", get().to(web::get::handler))
        .route("/{path}", post().to(web::post::handler))
}

/// Routes for api_v1
fn init_api_v1() -> impl HttpServiceFactory {
    use super::api::v1::*;
    scope("/api/v1")
        .service(index)
        .service(is_online)
        .service(online_users)
        .service(server_status)
        .service(verified_status)
        .service(ci_trigger)
        .service(bot_message)
        .service(update_user_stats)
}

/// Routes for api_v2
fn init_api_v2() -> impl HttpServiceFactory {
    use super::api::v2::*;
    scope("/api/v2").service(index)
}
