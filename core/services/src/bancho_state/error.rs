use crate::signature::error::SignatureError;
use peace_pb::ConvertError;
use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum CreateSessionError {
    #[error("invalid connection info")]
    InvalidConnectionInfo,
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum BanchoStateError {
    #[error("invalid argument")]
    InvalidArgument,
    #[error("bancho session not exists")]
    SessionNotExists,
    #[error(transparent)]
    SignatureError(#[from] SignatureError),
    #[error(transparent)]
    CreateSessionError(#[from] CreateSessionError),
    #[error(transparent)]
    ConvertError(#[from] ConvertError),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for BanchoStateError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}
