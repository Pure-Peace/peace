use super::{CHO_PROTOCOL, CHO_TOKEN};
use crate::{bancho::BanchoServiceError, bancho_state::BanchoStateError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bancho_packets::{server, PacketBuilder, PacketId};
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
    #[error(transparent)]
    SessionNotExists(BanchoStateError),
    #[error("unhandled packet: {0:?}")]
    UnhandledPacket(PacketId),
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
    #[error(transparent)]
    BanchoStateError(#[from] BanchoStateError),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl BanchoHttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseRequestError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::OK,
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
            Self::LoginFailed(_) => {
                (
                    [(CHO_TOKEN, "failed"), CHO_PROTOCOL],
                    PacketBuilder::new()
                        .add(server::login_reply(
                            bancho_packets::LoginResult::Failed(
                                bancho_packets::LoginFailedResaon::InvalidCredentials,
                            ),
                        ))
                        .add(server::notification(self.to_string()))
                        .build()
                )
                    .into_response()
            },

            Self::SessionNotExists(_) => {
                PacketBuilder::new().add(server::bancho_restart(0)).build().into_response()
            },

            _ => (self.status_code(), self.to_string()).into_response(),
        }
    }
}
