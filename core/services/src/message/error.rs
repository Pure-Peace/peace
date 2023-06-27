use async_nats::PublishError;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use tonic::{Code, Status};

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error(transparent)]
    PublishStreamError(Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    PublishError(#[from] PublishError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl MessageError {
    fn tonic_code(&self) -> Code {
        Code::Internal
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for MessageError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<MessageError> for Status {
    fn from(err: MessageError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}
