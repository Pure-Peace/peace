use ntex::web::{self, guard, scope, ServiceConfig};

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig, config_data: &peace_settings::local::LocalConfigData) {
    init_default(cfg);
    init_bancho(cfg);
    init_web(cfg);
    init_api_v1(cfg);
    init_api_v2(cfg);

    if config_data.geoip.web_api {
        init_geoip(cfg);
    };

    // !warning: only debug!
    if config_data.debug == true {
        init_debug(cfg)
    }
}

// Init geoip api
fn init_geoip(cfg: &mut ServiceConfig) {
    use super::geoip::*;
    cfg.service(scope("/geoip").service(index).service(geo_ip));
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
fn init_bancho(cfg: &mut ServiceConfig) {
    use super::bancho;
    cfg.service(
        scope("/bancho")
            .route("/", web::get().to(bancho::get::handler))
            .route(
                "/",
                web::post()
                    .guard(guard::Header("user-agent", "osu!"))
                    .to(bancho::post::handler),
            )
            .service(bancho::osu_register),
    );
}

/// Routes for web
fn init_web(cfg: &mut ServiceConfig) {
    use super::web::*;
    cfg.service(
        scope("/web")
            .route("/{path}", web::get().to(get::handler))
            .route("/{path}", web::post().to(post::handler)),
    );
}

/// Routes for api_v1
fn init_api_v1(cfg: &mut ServiceConfig) {
    use super::api::v1::*;
    cfg.service(
        scope("/api/v1")
            .service(index)
            .service(is_online)
            .service(online_users)
            .service(server_status)
            .service(verified_status)
            .service(ci_trigger)
            .service(bot_message)
            .route("/update_user_stats", web::get().to(update_user_stats))
            .route("/update_user_stats", web::post().to(update_user_stats))
            .service(recreate_score_table),
    );
}

/// Routes for api_v2
fn init_api_v2(cfg: &mut ServiceConfig) {
    use super::api::v2::*;
    cfg.service(scope("/api/v2").service(index));
}
