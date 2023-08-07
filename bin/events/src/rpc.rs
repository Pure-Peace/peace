use core_events::DynEventsService;
use pb_base::ExecSuccess;
use pb_events::*;
use std::pin::Pin;
use tonic::{codegen::futures_core::Stream, Request, Response, Status};

#[derive(Clone)]
pub struct EventsRpcImpl {
    pub events_service: DynEventsService,
}

impl EventsRpcImpl {
    pub fn new(events_service: DynEventsService) -> Self {
        Self { events_service }
    }
}

#[tonic::async_trait]
impl events_rpc_server::EventsRpc for EventsRpcImpl {
    type ConnectServerStream =
        Pin<Box<dyn Stream<Item = Result<Event, Status>> + Send>>;

    async fn connect_server(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<Self::ConnectServerStream>, Status> {
        /* let output_stream = ReceiverStream::new(rx); */
        todo!();

        /* Ok(Response::new(Box::pin(output_stream) as Self::ConnectServerStream)) */
    }
}
