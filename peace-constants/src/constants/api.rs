pub const OSU_FILE_DOWNLOAD_URL: &str = "https://old.ppy.sh/osu/";

#[derive(Debug)]
pub enum GetBeatmapMethod {
    Md5,
    Bid,
    Sid,
}

impl GetBeatmapMethod {
    #[inline(always)]
    pub fn db_column_name(&self) -> String {
        match self {
            &Self::Md5 => "md5",
            &Self::Bid => "id",
            &Self::Sid => "set_id",
        }
        .to_string()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ApiError {
    NotExists,
    RequestError,
    ParseError,
}
