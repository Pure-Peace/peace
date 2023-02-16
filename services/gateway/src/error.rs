use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("login: {0}")]
    Login(String),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Login(_) => StatusCode::UNAUTHORIZED,
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
        (self.status_code(), self.to_string()).into_response()
    }
}
