pub mod protobufs;
pub use protobufs::*;
use tonic::{Code, Status};

pub const OUT_DIR: &str = "generated";

#[derive(thiserror::Error, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvertError {
    #[error(transparent)]
    DecodingError(#[from] tools::DecodingError),
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
