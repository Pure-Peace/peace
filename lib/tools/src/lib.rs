#[cfg(feature = "async_collections")]
pub mod async_collections;

pub mod atomic;
#[cfg(feature = "cache")]
pub mod cache;
pub mod constants;
#[cfg(feature = "crypto")]
pub mod crypto;
pub mod macros;
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
