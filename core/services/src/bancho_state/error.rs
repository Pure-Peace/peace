use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tonic::{Code, Status};

#[derive(thiserror::Error, Debug)]
pub enum CreateSessionError {
    #[error("invalid connection info")]
    InvalidConnectionInfo,
}

#[derive(thiserror::Error, Debug)]
pub enum BanchoStateError {
    #[error("session not exists")]
    SessionNotExists,
    #[error("invalid bancho packet target")]
    BanchoPacketTarget,
    #[error(transparent)]
    CreateSessionError(#[from] CreateSessionError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl BanchoStateError {
    fn tonic_code(&self) -> Code {
        match self {
            Self::SessionNotExists => Code::NotFound,
            Self::BanchoPacketTarget => Code::InvalidArgument,
            _ => Code::Unknown,
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::OK,
        }
    }
}

impl IntoResponse for BanchoStateError {
    fn into_response(self) -> Response {
        match self {
            _ => (self.status_code(), self.to_string()).into_response(),
        }
    }
}

impl Into<Status> for BanchoStateError {
    fn into(self) -> Status {
        match self {
            _ => Status::new(self.tonic_code(), self.to_string()),
        }
    }
}
