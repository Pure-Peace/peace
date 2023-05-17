pub use rusty_ulid::{DecodingError, Ulid as RawUlid};
use std::{fmt::Display, str::FromStr};

#[cfg(feature = "async_collections")]
pub mod async_collections;

pub mod atomic;
#[cfg(feature = "cache")]
pub mod cache;
pub mod constants;
#[cfg(feature = "crypto")]
pub mod crypto;
pub mod macros;
pub mod message_queue;
#[cfg(feature = "tonic_utils")]
pub mod tonic_utils;

pub fn split_string(s: &str, sep: char) -> Vec<String> {
    s.trim()
        .split(sep)
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned())
        .collect::<Vec<String>>()
}

pub struct Timestamp;

impl Timestamp {
    pub fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_secs()
    }
}

#[derive(
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    derive_deref::Deref,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Ulid(RawUlid);

impl Ulid {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Display for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

impl FromStr for Ulid {
    type Err = DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(RawUlid::from_str(s)?))
    }
}

impl From<RawUlid> for Ulid {
    fn from(ulid: RawUlid) -> Self {
        Self(ulid)
    }
}

impl From<(u64, u64)> for Ulid {
    fn from(value: (u64, u64)) -> Self {
        Self(RawUlid::from(value))
    }
}

impl From<[u8; 16]> for Ulid {
    fn from(bytes: [u8; 16]) -> Self {
        Self(RawUlid::from(bytes))
    }
}

impl From<Ulid> for [u8; 16] {
    fn from(ulid: Ulid) -> Self {
        ulid.0.into()
    }
}

impl From<Ulid> for (u64, u64) {
    fn from(ulid: Ulid) -> Self {
        ulid.0.into()
    }
}

impl From<Ulid> for u128 {
    fn from(ulid: Ulid) -> Self {
        ulid.0.into()
    }
}

impl From<u128> for Ulid {
    fn from(value: u128) -> Self {
        Self(value.into())
    }
}

impl Default for Ulid {
    fn default() -> Self {
        Self(RawUlid::generate())
    }
}

impl message_queue::MessageId for Ulid {
    fn generate() -> Self {
        Self::new()
    }
}
