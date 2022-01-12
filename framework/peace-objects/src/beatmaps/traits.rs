#[cfg(feature = "async_trait")]
use async_trait::async_trait;

pub trait MyBeatmapCache<T> {
    fn new(beatmap: Option<T>) -> Self;
    fn is_expired(&self, expires: i64) -> bool;
}

#[cfg_attr(feature = "async_trait", async_trait)]
pub trait BeatmapCacheStorage<B: Clone, T> {
    #[cfg(feature = "async_trait")]
    async fn get(
        &self,
        md5: Option<&String>,
        bid: Option<i32>,
        sid: Option<i32>,
        file_name: Option<&String>,
    ) -> Option<T>;
    #[cfg(feature = "async_trait")]
    async fn cache(
        &self,
        md5: Option<&String>,
        bid: Option<i32>,
        sid: Option<i32>,
        file_name: Option<&String>,
        beatmap: Option<&B>,
    ) -> Option<T>;
    #[cfg(feature = "async_trait")]
    async fn cache_with_md5(&self, md5: &String, beatmap: Option<&B>) -> Option<T>;
    #[cfg(feature = "async_trait")]
    async fn cache_with_bid(&self, bid: i32, beatmap: Option<&B>) -> Option<T>;
    #[cfg(feature = "async_trait")]
    async fn cache_with_sid(&self, sid: i32, file_name: &String, beatmap: Option<&B>) -> Option<T>;
    #[cfg(feature = "async_trait")]
    async fn clean(&self) -> i32;
    #[cfg(feature = "async_trait")]
    async fn remove_timeouted(&self, expires: i64) -> i32;
}
