use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum EventsError {
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for EventsError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}
