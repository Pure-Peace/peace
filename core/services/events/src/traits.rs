use std::sync::Arc;

pub type DynEventsService = Arc<dyn EventsService + Send + Sync>;

pub trait EventsService {}
