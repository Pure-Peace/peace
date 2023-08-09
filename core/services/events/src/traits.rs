use crate::{EventsError, Subscription, SubscriptionWithOutputStream};
use async_trait::async_trait;
use pb_base::ExecSuccess;
use pb_events::Event;
use std::sync::Arc;

pub type DynEventsService = Arc<dyn EventsService + Send + Sync>;

#[async_trait]
pub trait EventsService {
    async fn create_subscription(
        &self,
        subscriber_key: String,
        buffer_server: usize,
        buffer_client: usize,
    ) -> Result<SubscriptionWithOutputStream<Event>, EventsError>;

    async fn remove_subscription(
        &self,
        subscriber_key: &String,
    ) -> Result<Option<Arc<Subscription<Event>>>, EventsError>;

    async fn publish(
        &self,
        subscriber_key: &String,
        event: Event,
    ) -> Result<ExecSuccess, EventsError>;
}
