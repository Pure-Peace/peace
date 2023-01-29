#[macro_use]
extern crate peace_logs;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate peace_api;

pub mod apidocs;
pub mod bancho;

use apidocs::GatewayApiDocs;
use axum::{
    async_trait,
    routing::{get, post},
    Router,
};
use bancho::handler;
use clap_serde_derive::ClapSerde;
use peace_api::{ApiFrameConfig, Application, RpcClientConfig};
use peace_pb::services::bancho_rpc;
use std::sync::Arc;
use utoipa::OpenApi;

define_rpc_client_config!(
    service_name: bancho_rpc,
    config_name: BanchoRpcConfig
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
        let bancho_rpc_client =
            self.cfg.bancho.connect_client().await.unwrap_or_else(|err| {
                error!("Unable to connect to the bancho gRPC service, please make sure the service is started.");
                panic!("{}", err)
            });

        Router::new()
            .route("/", get(handler::bancho_get))
            .route("/", post(handler::bancho_post))
            .route("/ss/:screenshot", get(handler::get_screenshot))
            .route("/d/:beatmapset_id", get(handler::download_beatmapset))
            .route("/users", post(handler::client_register))
            .route("/p/doyoureallywanttoaskpeppy", get(handler::ask_peppy))
            .route("/difficulty-rating", get(handler::difficulty_rating))
            .route("/web/osu-error.php", post(handler::osu_error))
            .route("/web/osu-screenshot.php", post(handler::osu_screenshot))
            .route("/web/osu-getfriends.php", get(handler::osu_getfriends))
            .route(
                "/web/osu-getbeatmapinfo.php",
                get(handler::osu_getbeatmapinfo),
            )
            .route(
                "/web/osu-getfavourites.php",
                get(handler::osu_getfavourites),
            )
            .route("/web/osu-addfavourite.php", get(handler::osu_addfavourite))
            .route("/web/lastfm.php", get(handler::lastfm))
            .route("/web/osu-search.php", get(handler::osu_search))
            .route("/web/osu-search-set.php", get(handler::osu_search_set))
            .route(
                "/web/osu-submit-modular-selector.php",
                post(handler::osu_submit_modular_selector),
            )
            .route("/web/osu-getreplay.php", get(handler::osu_getreplay))
            .route("/web/osu-rate.php", get(handler::osu_rate))
            .route(
                "/web/osu-osz2-getscores.php",
                get(handler::osu_osz2_getscores),
            )
            .route("/web/osu-comment.php", post(handler::osu_comment))
            .route("/web/osu-markasread.php", get(handler::osu_markasread))
            .route("/web/osu-getseasonal.php", get(handler::osu_getseasonal))
            .route("/web/bancho_connect.php", get(handler::bancho_connect))
            .route("/web/check-updates", get(handler::check_updates))
            .route("/web/maps/:beatmap_file_name", get(handler::update_beatmap))
            .route("/test", get(handler::test))
            .with_state(bancho_rpc_client)
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }
}
