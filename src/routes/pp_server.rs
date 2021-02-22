use actix_web::web::ServiceConfig;

use crate::settings::model::Settings;

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig, settings: Settings) {
    // !warning: only debug!
    if settings.debug == true {
        init_debug(cfg)
    }
}

fn init_debug(cfg: &mut ServiceConfig) {
    use super::debug::*;
    cfg.service(bancho_config_update);
    cfg.service(bancho_config_get);
    cfg.service(osu_api_test);
}
