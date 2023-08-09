use core_events::{
    DynEventsService, EventsError, SubscriptionWithOutputStream,
};
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
    type CreateSubscriptionStream =
        Pin<Box<dyn Stream<Item = Result<Event, Status>> + Send>>;

    async fn create_subscription(
        &self,
        request: Request<CreateSubscriptionRequest>,
    ) -> Result<Response<Self::CreateSubscriptionStream>, Status> {
        let CreateSubscriptionRequest { subscriber_key } = request.into_inner();

        let SubscriptionWithOutputStream { stream, .. } = self
            .events_service
            .create_subscription(subscriber_key, 1, 1)
            .await?;

        Ok(Response::new(Box::pin(stream) as Self::CreateSubscriptionStream))
    }

    async fn remove_subscription(
        &self,
        request: Request<RemoveSubscriptionRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let RemoveSubscriptionRequest { subscriber_key } = request.into_inner();

        self.events_service.remove_subscription(&subscriber_key).await?;

        Ok(Response::new(ExecSuccess::default()))
    }

    async fn publish(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<ExecSuccess>, Status> {
        let PublishRequest { subscriber_key, event } = request.into_inner();
        let event = event.ok_or(EventsError::InvalidArgument)?;

        self.events_service.publish(&subscriber_key, event).await?;

        Ok(Response::new(ExecSuccess::default()))
    }
}
