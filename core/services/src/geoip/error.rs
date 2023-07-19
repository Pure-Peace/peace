use peace_rpc_error::{RpcError, TonicError};
use tonic::Status;

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum GeoipError {
    #[error("geo-ip local was not initialized")]
    NotInitialized,
    #[error("failed to lookup ip address: {0}")]
    LookupError(String),
    #[error("this only for local service")]
    OnlyLocalService,
    #[error("failed to load geo-ip database: {0}")]
    FailedToLoadDatabase(String),
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl TonicError for GeoipError {
    fn tonic_error(s: Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}
