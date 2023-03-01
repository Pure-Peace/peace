use super::{CHO_PROTOCOL, CHO_TOKEN};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bancho_packets::{server, PacketBuilder};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Login failed: {0}")]
    Login(String),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Login(_) => StatusCode::OK,
        }
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        err.into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Login(_) => (
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
