use axum::response::{IntoResponse, Response};
use hex::FromHexError;
use hyper::StatusCode;
use peace_rpc::RpcError;
use tonic::Code;
use tools::crypto::Ed25519Error;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum SignatureError {
    #[error(transparent)]
    Ed25519Error(#[from] Ed25519Error),
    #[error("DecodeHexError: {0}")]
    DecodeHexError(String),
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

impl From<FromHexError> for SignatureError {
    fn from(err: FromHexError) -> Self {
        Self::DecodeHexError(err.to_string())
    }
}

/* impl TryFrom<Status> for SignatureError {
    type Error;

    fn try_from(value: Status) -> Result<Self, Self::Error> {
        todo!()
    }
} */
