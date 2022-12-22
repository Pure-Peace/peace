pub mod apidocs;
pub mod bancho;
pub mod cfg;

use apidocs::GatewayApiDocs;
use axum::{
    async_trait, body::Body, extract::Host, http::Request, routing::get, Router,
};
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
        let bancho_client = self.connect_bancho_service().await.unwrap();

        Router::new()
            .route("/", get(peace_api::responder::app_root))
            .nest("/bancho", bancho::routers::bancho_client_routes())
            .with_state(bancho_client)
    }

    fn apidocs(&self) -> utoipa::openapi::OpenApi {
        GatewayApiDocs::openapi()
    }

    async fn match_hostname(
        &self,
        Host(hostname): Host,
        req: &Request<Body>,
    ) -> Option<Router> {
        match hostname {
            n if self.cfg.bancho_hostname.contains(&n) => {
                Some(bancho::routers::bancho_client_routes())
            },
            _ => None,
        }
    }
}
