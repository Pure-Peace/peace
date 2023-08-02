pub use rusty_ulid as raw;
use std::{fmt::Display, str::FromStr};

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
pub struct Ulid(raw::Ulid);

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
    type Err = raw::DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(raw::Ulid::from_str(s)?))
    }
}

impl From<raw::Ulid> for Ulid {
    fn from(ulid: raw::Ulid) -> Self {
        Self(ulid)
    }
}

impl From<(u64, u64)> for Ulid {
    fn from(value: (u64, u64)) -> Self {
        Self(raw::Ulid::from(value))
    }
}

impl From<[u8; 16]> for Ulid {
    fn from(bytes: [u8; 16]) -> Self {
        Self(raw::Ulid::from(bytes))
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
        Self(raw::Ulid::generate())
    }
}

#[cfg(feature = "message_id")]
impl peace_message_queue::MessageId for Ulid {
    fn generate() -> Self {
        Self::new()
    }
}
