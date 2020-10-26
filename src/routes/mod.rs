use actix_web::dev::HttpServiceFactory;
use actix_web::web::{scope, ServiceConfig};

pub mod api_v1;
pub mod api_v2;
pub mod bancho;

/// Function that will be called on new Application to configure routes for this module
/// Initial all routes
pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(init_bancho());
    cfg.service(init_api_v1());
    cfg.service(init_api_v2());
}

/// Routes for bancho
fn init_bancho() -> impl HttpServiceFactory {
    use bancho::*;
    scope("/")
        .service(main)
}

/// Routes for api_v1
fn init_api_v1() -> impl HttpServiceFactory {
    use api_v1::*;
    scope("/api/v1")
        .service(index)
}

/// Routes for api_v2
fn init_api_v2() -> impl HttpServiceFactory {
    use api_v2::*;
    scope("/api/v2")
        .service(index)
}
