#[macro_use]
extern crate peace_logs;

pub mod apidocs;
pub mod bancho;
pub mod cfg;

use apidocs::GatewayApiDocs;
use axum::{
    async_trait,
    routing::{get, post},
    Router,
};
use bancho::handler;
use cfg::GatewayConfig;
use peace_api::{cfg::ApiFrameConfig, Application};
use peace_pb::services::bancho::bancho_rpc_client::BanchoRpcClient;
use std::sync::Arc;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use utoipa::OpenApi;

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<GatewayConfig>,
}

impl App {
    pub fn new(cfg: Arc<GatewayConfig>) -> Self {
        Self { cfg }
    }

    pub async fn connect_bancho_service(
        &self,
    ) -> Result<BanchoRpcClient<Channel>, anyhow::Error> {
        info!("Connecting bancho gRPC service...");
        #[cfg(unix)]
        if let Some(uds) = self.cfg.bancho_uds {
            let channel =
                tonic::transport::Endpoint::try_from("http://[::]:50051")?
                    .connect_with_connector(tower::service_fn(|_| {
                        tokio::net::UnixStream::connect(uds)
                    }))
                    .await?;
            return BanchoRpcClient::new(channel);
        }

        if self.cfg.bancho_tls {
            let pem =
                tokio::fs::read(self.cfg.bancho_ssl_cert.as_ref().unwrap())
                    .await?;
            let ca = Certificate::from_pem(pem);
            let tls = ClientTlsConfig::new().ca_certificate(ca);
            let channel = Channel::from_shared(self.cfg.bancho_addr.clone())?
                .tls_config(tls)?
                .connect()
                .await?;

            return Ok(BanchoRpcClient::new(channel));
        }

        Ok(BanchoRpcClient::connect(self.cfg.bancho_addr.clone()).await?)
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
    }

    async fn router<T: Clone + Sync + Send + 'static>(&self) -> Router<T> {
        let bancho_handlers = self.connect_bancho_service().await.unwrap();

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
            .with_state(bancho_handlers)
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }
}
