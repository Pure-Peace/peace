pub mod protobufs;
pub use protobufs::*;

use peace_rpc_error::RpcError;
use serde::{Deserialize, Serialize};
use tools::DecodingError;

pub const OUT_DIR: &str = "generated";

#[derive(thiserror::Error, Debug, Serialize, Deserialize, RpcError)]
pub enum ConvertError {
    #[error("ulid decoding error: {0}")]
    DecodingError(String),
    #[error("invalid params")]
    InvalidParams,
    #[error("from Channel target is not support")]
    FromChannelTarget,
    #[error("TonicError: {0}")]
    TonicError(String),
}

impl peace_rpc_error::TonicError for ConvertError {
    fn tonic_error(s: tonic::Status) -> Self {
        Self::TonicError(s.message().to_owned())
    }
}

impl From<DecodingError> for ConvertError {
    fn from(err: DecodingError) -> Self {
        Self::DecodingError(err.to_string())
    }
}
