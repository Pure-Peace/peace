mod api;
mod debug;
mod default;

use ntex::web::{scope, ServiceConfig};

use crate::settings::model::LocalConfigData;

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig, settings: &LocalConfigData) {
    init_default(cfg);
    init_api(cfg);

    // !warning: only debug!
    if settings.debug == true {
        init_debug(cfg)
    }
}

/// Routes for api
fn init_api(cfg: &mut ServiceConfig) {
    use api::*;
    cfg.service(scope("/api").service(index).service(calculate_pp));
}

fn init_debug(cfg: &mut ServiceConfig) {
    use debug::*;
    cfg.service(index);
    cfg.service(server_stop);
    cfg.service(clear_cache);
}

/// Routes for default
fn init_default(cfg: &mut ServiceConfig) {
    use default::*;
    cfg.service(index);
}
