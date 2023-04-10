use crate::bancho_state::BanchoStateError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bancho_packets::PacketId;
use peace_domain::users::PasswordError;
use peace_pb::ConvertError;
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
pub enum ProcessBanchoPacketError {
    #[error("failed to process all bancho packets")]
    FailedToProcessAll,
    #[error("invalid packet id")]
    InvalidPacketId,
    #[error("packet payload not exists")]
    PacketPayloadNotExists,
    #[error("invalid packet payload")]
    InvalidPacketPayload,
    #[error("unhandled packet: {0:?}")]
    UnhandledPacket(PacketId),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl ProcessBanchoPacketError {
    fn tonic_code(&self) -> Code {
        match self {
            Self::FailedToProcessAll => Code::Internal,
            _ => Code::Unknown,
        }
    }
}

impl From<ProcessBanchoPacketError> for Status {
    fn from(err: ProcessBanchoPacketError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
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
    #[error(transparent)]
    ConvertError(#[from] ConvertError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl BanchoServiceError {
    fn tonic_code(&self) -> Code {
        match self {
            Self::SessionNotExists => Code::NotFound,
            Self::BanchoPacketTarget => Code::InvalidArgument,
            Self::ConvertError(_) => Code::InvalidArgument,
            _ => Code::Unknown,
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl IntoResponse for BanchoServiceError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<BanchoServiceError> for Status {
    fn from(err: BanchoServiceError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}
