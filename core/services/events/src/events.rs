use crate::*;
use async_trait::async_trait;
use infra_services::{FromRpcClient, IntoService, RpcClient};
use std::sync::Arc;
use tonic::transport::Channel;

#[derive(Clone, Default)]
pub struct EventsServiceImpl {}

impl EventsServiceImpl {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }
}

impl EventsService for EventsServiceImpl {}

impl IntoService<DynEventsService> for EventsServiceImpl {
    #[inline]
    fn into_service(self) -> DynEventsService {
        Arc::new(self) as DynEventsService
    }
}

/* #[derive(Debug, Clone)]
pub struct EventsServiceRemote(EventsRpcClient<Channel>);

impl RpcClient for EventsServiceRemote {
    type Client = EventsRpcClient<Channel>;

    #[inline]
    fn client(&self) -> Self::Client {
        self.0.clone()
    }
}

impl EventsService for EventsServiceRemote {}

impl IntoService<DynEventsService> for EventsServiceRemote {
    #[inline]
    fn into_service(self) -> DynEventsService {
        Arc::new(self) as DynEventsService
    }
}
 */
