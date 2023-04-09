#[cfg(feature = "async_collections")]
pub mod async_collections;

pub mod atomic;
#[cfg(feature = "cache")]
pub mod cache;
pub mod constants;
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
pub struct Ulid(rusty_ulid::Ulid);

impl Ulid {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Ulid {
    fn default() -> Self {
        Self(rusty_ulid::Ulid::generate())
    }
}

impl message_queue::MessageId for Ulid {
    fn generate() -> Self {
        Self::new()
    }
}
