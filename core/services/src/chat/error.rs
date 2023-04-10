use crate::bancho_state::BanchoStateError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use peace_pb::ConvertError;
use tonic::{Code, Status};

#[derive(thiserror::Error, Debug)]
pub enum ChatServiceError {
    #[error("invalid argument")]
    InvalidArgument,
    #[error("channel not exists")]
    ChannelNotExists,
    #[error(transparent)]
    ConvertError(#[from] ConvertError),
    #[error("bancho state error: {0}")]
    BanchoStateError(#[from] BanchoStateError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl ChatServiceError {
    fn tonic_code(&self) -> Code {
        match self {
            Self::ChannelNotExists => Code::NotFound,
            Self::InvalidArgument => Code::InvalidArgument,
            Self::ConvertError(_) => Code::InvalidArgument,
            _ => Code::Unknown,
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl IntoResponse for ChatServiceError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<ChatServiceError> for Status {
    fn from(err: ChatServiceError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}
