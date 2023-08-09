use crate::EventsRpcImpl;
use clap_serde_derive::ClapSerde;
use core_events::*;
use infra_services::IntoService;
use pb_events::{events_rpc_server::EventsRpcServer, EVENTS_DESCRIPTOR_SET};
use peace_rpc::{RpcApplication, RpcFrameConfig};
use peace_runtime::cfg::RuntimeConfig;
use std::{net::SocketAddr, sync::Arc};
use tonic::{
    async_trait,
    transport::{server::Router, Server},
};

/// PEACE Events service
#[peace_config]
#[command(name = "events", author, version, about, propagate_version = true)]
pub struct EventsConfig {
    #[command(flatten)]
    pub runtime_cfg: RuntimeConfig,

    #[command(flatten)]
    pub frame_cfg: RpcFrameConfig,
}

#[derive(Clone)]
pub struct App {
    pub cfg: Arc<EventsConfig>,
    pub events_service: DynEventsService,
    pub events_rpc: EventsRpcImpl,
}

impl App {
    pub async fn initialize(cfg: Arc<EventsConfig>) -> Self {
        let events_service = EventsServiceImpl::new().into_service();

        let events_rpc = EventsRpcImpl::new(events_service.clone());

        Self { cfg, events_service, events_rpc }
    }
}

#[async_trait]
impl RpcApplication for App {
    fn frame_cfg(&self) -> &RpcFrameConfig {
        &self.cfg.frame_cfg
    }

    fn default_listen_addr(&self) -> Option<SocketAddr> {
        Some("127.0.0.1:5015".parse().unwrap())
    }

    fn service_descriptors(&self) -> Option<&[&[u8]]> {
        Some(&[EVENTS_DESCRIPTOR_SET])
    }

    async fn service(&self, mut configured_server: Server) -> Router {
        configured_server
            .add_service(EventsRpcServer::new(self.events_rpc.clone()))
    }
}
