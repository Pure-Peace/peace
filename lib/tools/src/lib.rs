#[cfg(feature = "async_collections")]
pub mod async_collections;

pub mod constants;
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
