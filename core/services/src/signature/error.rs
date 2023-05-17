use axum::response::{IntoResponse, Response};
use hex::FromHexError;
use hyper::StatusCode;
use tonic::{Code, Status};
use tools::crypto::Ed25519Error;

#[derive(thiserror::Error, Debug)]
pub enum SignatureError {
    #[error(transparent)]
    Ed25519Error(#[from] Ed25519Error),
    #[error(transparent)]
    DecodeHexError(#[from] FromHexError),
    #[error("{}", .0.message())]
    RpcError(#[from] Status),
}

impl SignatureError {
    fn tonic_code(&self) -> Code {
        Code::Internal
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for SignatureError {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<SignatureError> for Status {
    fn from(err: SignatureError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}
