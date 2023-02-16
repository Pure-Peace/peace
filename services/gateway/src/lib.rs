#[macro_use]
extern crate peace_logs;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate peace_api;

pub mod apidocs;
pub mod bancho;
pub mod utils;

use apidocs::GatewayApiDocs;
use axum::{
    async_trait,
    routing::{get, post},
    Extension, Router,
};
use bancho::routes;
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application, RpcClientConfig};
use peace_pb::services::{bancho_rpc, bancho_state_rpc};
use std::sync::Arc;
use utoipa::OpenApi;

define_rpc_client_config!(
    service_name: bancho_rpc,
    config_name: BanchoRpcConfig
);

define_rpc_client_config!(
    service_name: bancho_state_rpc,
    config_name: BanchoStateRpcConfig,
    default_uri: "http://127.0.0.1:12345"
);

/// Command Line Interface (CLI) for Peace gateway service.
#[peace_config]
#[command(
    name = "peace-gateway",
    author,
    version,
    about,
    propagate_version = true
)]
pub struct GatewayConfig {
    #[command(flatten)]
    pub frame_cfg: ApiFrameConfig,

    #[command(flatten)]
    pub bancho: BanchoRpcConfig,

    #[command(flatten)]
    pub bancho_state: BanchoStateRpcConfig,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<GatewayConfig>,
}

impl App {
    pub fn new(cfg: Arc<GatewayConfig>) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
    }

    async fn router<T: Clone + Sync + Send + 'static>(&self) -> Router<T> {
        let bancho_rpc_client = self.cfg.bancho.connect_client().await.unwrap_or_else(|err| {
                error!("Unable to connect to the {err} service, please make sure the service is started.");
                panic!("{}", err)
            });

        let bancho_state_rpc_client =
        self.cfg.bancho_state.connect_client().await.unwrap_or_else(
                |err| {
                    error!("Unable to connect to the {err} service, please make sure the service is started.");
                    panic!("{}", err)
                },
            );

        Router::new()
            .route("/", get(routes::bancho_get))
            .route("/", post(routes::bancho_post))
            .route("/ss/:screenshot", get(routes::get_screenshot))
            .route("/d/:beatmapset_id", get(routes::download_beatmapset))
            .route("/users", post(routes::client_register))
            .route("/p/doyoureallywanttoaskpeppy", get(routes::ask_peppy))
            .route("/difficulty-rating", get(routes::difficulty_rating))
            .route("/web/osu-error.php", post(routes::osu_error))
            .route("/web/osu-screenshot.php", post(routes::osu_screenshot))
            .route("/web/osu-getfriends.php", get(routes::osu_getfriends))
            .route(
                "/web/osu-getbeatmapinfo.php",
                get(routes::osu_getbeatmapinfo),
            )
            .route("/web/osu-getfavourites.php", get(routes::osu_getfavourites))
            .route("/web/osu-addfavourite.php", get(routes::osu_addfavourite))
            .route("/web/lastfm.php", get(routes::lastfm))
            .route("/web/osu-search.php", get(routes::osu_search))
            .route("/web/osu-search-set.php", get(routes::osu_search_set))
            .route(
                "/web/osu-submit-modular-selector.php",
                post(routes::osu_submit_modular_selector),
            )
            .route("/web/osu-getreplay.php", get(routes::osu_getreplay))
            .route("/web/osu-rate.php", get(routes::osu_rate))
            .route(
                "/web/osu-osz2-getscores.php",
                get(routes::osu_osz2_getscores),
            )
            .route("/web/osu-comment.php", post(routes::osu_comment))
            .route("/web/osu-markasread.php", get(routes::osu_markasread))
            .route("/web/osu-getseasonal.php", get(routes::osu_getseasonal))
            .route("/web/bancho_connect.php", get(routes::bancho_connect))
            .route("/web/check-updates", get(routes::check_updates))
            .route("/web/maps/:beatmap_file_name", get(routes::update_beatmap))
            .route("/test", get(routes::test))
            .layer(Extension(bancho_rpc_client))
            .layer(Extension(bancho_state_rpc_client))
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }
}
