use super::{CHO_PROTOCOL, CHO_TOKEN};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bancho_packets::{server, PacketBuilder};
use tonic::Status;

#[derive(thiserror::Error, Debug)]
pub enum BanchoError {
    #[error("login failed: {0}")]
    Login(String),
    #[error("unhandled packet: {0}")]
    UnhandledPacket(String),
    #[error("failed to handling packet: {source}")]
    PacketHandlingError {
        #[source]
        source: Status,
    },
    #[error("failed to dequeue packets: {source}")]
    DequeuePakcetsError {
        #[source]
        source: Status,
    },
}

impl BanchoError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Login(_) => StatusCode::OK,
            Self::UnhandledPacket(_) => StatusCode::OK,
            Self::PacketHandlingError { .. } => StatusCode::OK,
            Self::DequeuePakcetsError { .. } => StatusCode::OK,
        }
    }
}

impl From<BanchoError> for Response {
    fn from(err: BanchoError) -> Self {
        err.into_response()
    }
}

impl IntoResponse for BanchoError {
    fn into_response(self) -> Response {
        match self {
            BanchoError::Login(_) => (
                [(CHO_TOKEN, "failed"), CHO_PROTOCOL],
                PacketBuilder::new()
                    .add(server::login_reply(
                        bancho_packets::LoginResult::Failed(
                            bancho_packets::LoginFailedResaon::InvalidCredentials,
                        ),
                    ))
                    .add(server::notification(self.to_string()))
                    .build(),
            )
                .into_response(),

            _ => (self.status_code(), self.to_string()).into_response(),
        }
    }
}
