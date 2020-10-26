use actix_web::web::ServiceConfig;

pub mod common;

/// Function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(common::index);
}
