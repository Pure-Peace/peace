use super::{CHO_PROTOCOL, CHO_TOKEN};
use crate::{
    bancho::{BanchoServiceError, ProcessBanchoPacketError},
    bancho_state::BanchoStateError,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bancho_packets::{server, PacketBuilder};
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum ParseLoginDataError {
    #[error("invalid request body")]
    InvalidRequestBody(#[source] FromUtf8Error),
    #[error("invalid login data")]
    InvalidLoginData,
    #[error("invalid user info")]
    InvalidUserInfo,
    #[error("invalid client info")]
    InvalidClientInfo,
    #[error("invalid client hashes")]
    InvalidClientHashes,
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("client version is empty")]
    EmptyClientVersion,
    #[error("mismatched client version")]
    MismatchedClientVersion,
    #[error(transparent)]
    ParseLoginDataError(#[from] ParseLoginDataError),
    #[error(transparent)]
    BanchoServiceError(#[from] BanchoServiceError),
}

#[derive(thiserror::Error, Debug)]
pub enum BanchoHttpError {
    #[error(transparent)]
    LoginFailed(#[from] LoginError),
    #[error("errors occured while handling packet: {0}")]
    PacketHandlingError(#[source] anyhow::Error),
    #[error("errors occured while dequeueing packets: {0}")]
    DequeuePakcetsError(#[source] BanchoStateError),
    #[error("invalid `osu-version` header")]
    InvalidOsuVersionHeader,
    #[error("invalid `osu-token` header")]
    InvalidOsuTokenHeader,
    #[error("invalid `user-agent` header")]
    InvalidUserAgentHeader,
    #[error("failed to parse request")]
    ParseRequestError,
    #[error("invalid bancho packet")]
    InvalidBanchoPacket,
    #[error("failed to process bancho packets")]
    FailedToProcessBanchoPackets(#[from] ProcessBanchoPacketError),
    #[error(transparent)]
    BanchoStateError(#[from] BanchoStateError),
}

impl BanchoHttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<BanchoHttpError> for Response {
    fn from(err: BanchoHttpError) -> Self {
        err.into_response()
    }
}

impl IntoResponse for BanchoHttpError {
    fn into_response(self) -> Response {
        match self {
            Self::LoginFailed(err) => {
                let login_reply =
                    server::LoginReply::failed_invalid_credentials();

                let packets = PacketBuilder::new()
                    .add(login_reply)
                    .add(server::Notification::new(err.to_string().into()))
                    .build();

                ([(CHO_TOKEN, "failed"), CHO_PROTOCOL], packets).into_response()
            },

            Self::BanchoStateError(
                BanchoStateError::SessionNotExists
                | BanchoStateError::SignatureError(_),
            ) => {
                (StatusCode::OK, server::BanchoRestart::pack(0)).into_response()
            },

            _ => {
                warn!("[BanchoHttpError] Unhandled error: {self:?}");
                (self.status_code(), self.to_string()).into_response()
            },
        }
    }
}
