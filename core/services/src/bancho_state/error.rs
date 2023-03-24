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
    #[error("invalid argument")]
    InvalidArgument,
    #[error("session not exists")]
    SessionNotExists,
    #[error(transparent)]
    CreateSessionError(#[from] CreateSessionError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl BanchoStateError {
    fn tonic_code(&self) -> Code {
        match self {
            Self::SessionNotExists => Code::NotFound,
            Self::InvalidArgument => Code::InvalidArgument,
            _ => Code::Unknown,
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl IntoResponse for BanchoStateError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<BanchoStateError> for Status {
    fn from(err: BanchoStateError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}
