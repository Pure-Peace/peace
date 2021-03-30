use super::requester::OsuApi;
use super::{depends::*, errors::ApiError};

impl OsuApi {
    #[inline(always)]
    pub async fn fetch_beatmap<Q: Serialize + ?Sized>(
        &self,
        query: &Q,
    ) -> Result<BeatmapFromApi, ApiError> {
        match self.fetch_beatmap_list(query).await?.pop() {
            Some(b) => Ok(b),
            None => Err(ApiError::NotExists),
        }
    }

    #[inline(always)]
    pub async fn fetch_beatmap_list<Q: Serialize + ?Sized>(
        &self,
        query: &Q,
    ) -> Result<Vec<BeatmapFromApi>, ApiError> {
        Ok(self
            .get_json::<_, Vec<BeatmapFromApi>>("https://old.ppy.sh/api/get_beatmaps", query)
            .await?)
    }

    #[inline(always)]
    pub async fn fetch_beatmap_by_md5(
        &self,
        beatmap_hash: &String,
    ) -> Result<BeatmapFromApi, ApiError> {
        self.fetch_beatmap(&[("h", beatmap_hash)]).await
    }

    #[inline(always)]
    pub async fn fetch_beatmap_by_bid(&self, beatmap_id: i32) -> Result<BeatmapFromApi, ApiError> {
        self.fetch_beatmap(&[("b", beatmap_id)]).await
    }

    #[inline(always)]
    pub async fn fetch_beatmap_by_sid(
        &self,
        beatmap_set_id: i32,
    ) -> Result<Vec<BeatmapFromApi>, ApiError> {
        self.fetch_beatmap_list(&[("s", beatmap_set_id)]).await
    }

    #[inline(always)]
    pub async fn fetch_beatmap_by(
        &self,
        key: &str,
        value: &String,
    ) -> Result<Vec<BeatmapFromApi>, ApiError> {
        self.fetch_beatmap_list(&[(key, value)]).await
    }

    #[inline(always)]
    pub async fn fetch_beatmap_by_uid(
        &self,
        user_id: i32,
    ) -> Result<Vec<BeatmapFromApi>, ApiError> {
        self.fetch_beatmap_list(&[("u", user_id)]).await
    }
}
