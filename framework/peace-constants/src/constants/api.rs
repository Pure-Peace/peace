use serde::{Deserialize, Serialize};

pub const OSU_FILE_DOWNLOAD_URL: &str = "https://old.ppy.sh/osu/";

#[derive(Debug)]
pub enum GetBeatmapMethod {
    Md5(String),
    Bid(i32),
    Sid(i32),
}

impl GetBeatmapMethod {
    #[inline]
    pub fn db_column_name(&self) -> String {
        match self {
            &Self::Md5(_) => "md5",
            &Self::Bid(_) => "id",
            &Self::Sid(_) => "set_id",
        }
        .to_string()
    }

    #[inline]
    pub fn to_string(&self) -> String {
        match self {
            Self::Md5(v) => v.to_string(),
            Self::Bid(v) => v.to_string(),
            Self::Sid(v) => v.to_string(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ApiError {
    NotExists,
    RequestError,
    ParseError,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UpdateUserTask {
    pub player_id: i32,
    pub mode: u8,
    pub recalc: bool,
}
