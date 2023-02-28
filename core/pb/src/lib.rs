pub mod protobufs;
pub use protobufs::*;

pub const OUT_DIR: &str = "generated";

#[derive(Debug, Clone)]
pub struct ConvertError(String);

impl ConvertError {
    pub fn new<D: ::core::fmt::Display>(raw: D) -> Self {
        Self(raw.to_string())
    }
}

impl ToString for ConvertError {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<ConvertError> for String {
    fn from(err: ConvertError) -> Self {
        err.0
    }
}
