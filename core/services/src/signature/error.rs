use hex::FromHexError;
use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;
use tools::crypto::Ed25519Error;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum SignatureError {
    #[error(transparent)]
    Ed25519Error(#[from] Ed25519Error),
    #[error("DecodeHexError: {0}")]
    DecodeHexError(String),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for SignatureError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}

impl From<FromHexError> for SignatureError {
    fn from(err: FromHexError) -> Self {
        Self::DecodeHexError(err.to_string())
    }
}
