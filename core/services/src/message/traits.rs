use super::error::MessageError;
use async_nats::jetstream::context::PublishAckFuture;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;

pub type DynMessageService = Arc<dyn MessageService + Send + Sync>;

pub trait MessageService: PublishMessage {}

#[async_trait]
pub trait PublishMessage {
    async fn publish<'a>(
        &self,
        subject: String,
        payload: Bytes,
    ) -> Result<(), MessageError>;

    async fn publish_stream<'a>(
        &self,
        subject: String,
        payload: Bytes,
    ) -> Result<PublishAckFuture, MessageError>;
}
