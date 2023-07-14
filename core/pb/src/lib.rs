pub mod protobufs;
pub use protobufs::*;

use serde::{Deserialize, Serialize};
use tonic::{Code, Status};
use tools::DecodingError;

pub const OUT_DIR: &str = "generated";

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum ConvertError {
    #[error("ulid decoding error: {0}")]
    DecodingError(String),
    #[error("invalid params")]
    InvalidParams,
    #[error("from Channel target is not support")]
    FromChannelTarget,
}

impl ConvertError {
    fn tonic_code(&self) -> Code {
        Code::InvalidArgument
    }
}

impl From<ConvertError> for Status {
    fn from(err: ConvertError) -> Self {
        Status::new(err.tonic_code(), err.to_string())
    }
}

impl From<DecodingError> for ConvertError {
    fn from(err: DecodingError) -> Self {
        Self::DecodingError(err.to_string())
    }
}
