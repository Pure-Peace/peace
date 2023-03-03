use crate::bancho_state::BanchoStateError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use peace_domain::users::PasswordError;
use peace_repositories::GetUserError;
use tonic::{Code, Status};

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error(transparent)]
    PasswordError(#[from] PasswordError),
    #[error(transparent)]
    UserNotExists(#[from] GetUserError),
}

#[derive(thiserror::Error, Debug)]
pub enum BanchoServiceError {
    #[error("session not exists")]
    SessionNotExists,
    #[error("invalid bancho packet target")]
    BanchoPacketTarget,
    #[error(transparent)]
    LoginError(#[from] LoginError),
    #[error(transparent)]
    BanchoStateError(#[from] BanchoStateError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl BanchoServiceError {
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

impl IntoResponse for BanchoServiceError {
    fn into_response(self) -> Response {
        match self {
            _ => (self.status_code(), self.to_string()).into_response(),
        }
    }
}

impl Into<Status> for BanchoServiceError {
    fn into(self) -> Status {
        match self {
            _ => Status::new(self.tonic_code(), self.to_string()),
        }
    }
}
