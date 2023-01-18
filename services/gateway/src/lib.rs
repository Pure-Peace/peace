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
use peace_api::{ApiFrameConfig, Application};
use peace_pb::services::bancho::bancho_rpc_client::BanchoRpcClient;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};
use utoipa::OpenApi;

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

    /// A list of hostnames to route to the bancho service.
    #[arg(short = 'B', long)]
    pub bancho_hostname: Vec<String>,

    /// Bancho service address.
    #[default("http://127.0.0.1:50051".to_owned())]
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    pub bancho_addr: String,

    /// Bancho service unix domain socket path.
    /// Only for unix systems.
    ///
    /// `uds` will be preferred over `addr`.
    #[arg(long)]
    pub bancho_uds: Option<PathBuf>,

    /// `tls` connection for Bancho service.
    #[default(false)]
    #[arg(long)]
    pub bancho_tls: bool,

    /// SSL certificate path.
    #[arg(long)]
    pub bancho_ssl_cert: Option<PathBuf>,

    /// Not attempt to connect to the bancho endpoint until first use.
    #[default(false)]
    #[arg(long)]
    pub bancho_lazy_connect: bool,
}

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
        async fn connect_endpoint(
            endpoint: Endpoint,
            lazy_connect: bool,
        ) -> Result<Channel, anyhow::Error> {
            Ok(if lazy_connect {
                endpoint.connect_lazy()
            } else {
                info!("Attempting to connect to bancho gRPC endpoint...");
                endpoint.connect().await?
            })
        }

        #[cfg(unix)]
        if let Some(uds) = self.cfg.bancho_uds {
            info!("Bancho gRPC service: {}", uds);
            let service_factory =
                tower::service_fn(|_| tokio::net::UnixStream::connect(uds));
            let endpoint =
                tonic::transport::Endpoint::try_from("http://[::]:50051")?;

            let channel = if self.cfg.bancho_lazy_connect {
                endpoint.connect_with_connector_lazy(service_factory)
            } else {
                info!("Attempting to connect to bancho gRPC endpoint...");
                endpoint.connect_with_connector(service_factory).await?
            };
            return BanchoRpcClient::new(channel);
        }

        info!("Bancho gRPC service: {}", self.cfg.bancho_addr);
        if self.cfg.bancho_tls {
            let pem =
                tokio::fs::read(self.cfg.bancho_ssl_cert.as_ref().unwrap())
                    .await?;
            let ca = Certificate::from_pem(pem);
            let tls = ClientTlsConfig::new().ca_certificate(ca);
            return Ok(BanchoRpcClient::new(
                connect_endpoint(
                    Channel::from_shared(self.cfg.bancho_addr.clone())?
                        .tls_config(tls)?,
                    self.cfg.bancho_lazy_connect,
                )
                .await?,
            ));
        }

        Ok(BanchoRpcClient::new(
            connect_endpoint(
                Channel::from_shared(self.cfg.bancho_addr.clone())?,
                self.cfg.bancho_lazy_connect,
            )
            .await?,
        ))
    }
}

#[async_trait]
impl Application for App {
    fn frame_cfg(&self) -> &ApiFrameConfig {
        &self.cfg.frame_cfg
    }

    async fn router<T: Clone + Sync + Send + 'static>(&self) -> Router<T> {
        let bancho_handlers =
            self.connect_bancho_service().await.unwrap_or_else(|err| {
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
            .with_state(bancho_handlers)
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }
}
