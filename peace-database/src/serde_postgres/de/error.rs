//! When serializing or deserializing from Postgres rows goes wrong.
use serde::de;
use std::{error, fmt};

/// Alias for a `Result` with the error type `serde_postgres::Error`.
pub type DeResult<T> = ::std::result::Result<T, DeError>;

/// This type represents all possible error that can occur when deserializing
/// postgres rows.
#[derive(Clone, Debug, PartialEq)]
pub enum DeError {
    /// A custom defined error occurred. Typically coming from `serde`.
    Message(String),
    /// Row contained a field unknown to the data structure.
    UnknownField,
    /// Row's column type was different from the Rust data structure.
    InvalidType(String),
    /// Rust data structure contained a type unsupported by `serde_postgres`.
    UnsupportedType,
}

impl de::Error for DeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeError::Message(msg.to_string())
    }
}

impl fmt::Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            DeError::Message(ref msg) => msg,
            DeError::UnknownField => "Unknown field",
            DeError::InvalidType(_) => "Invalid type",
            DeError::UnsupportedType => "Type unsupported",
        };
        f.write_str(msg)
    }
}

impl error::Error for DeError {}
