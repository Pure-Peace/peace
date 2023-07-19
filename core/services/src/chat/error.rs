use crate::bancho_state::BanchoStateError;
use peace_pb::ConvertError;
use peace_repositories::GetUserError;
use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum ChatError {
    #[error(transparent)]
    GetUserError(#[from] GetUserError),
    #[error("invalid argument")]
    InvalidArgument,
    #[error("session not exists")]
    SessionNotExists,
    #[error("channel not exists")]
    ChannelNotExists,
    #[error(transparent)]
    ConvertError(#[from] ConvertError),
    #[error("bancho state error: {0}")]
    BanchoStateError(#[from] BanchoStateError),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for ChatError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}
