use {
    enum_primitive_derive::Primitive,
    num_traits::FromPrimitive,
    serde::{de::Error, Deserialize, Deserializer},
};

pub const CHEAT_DETECTED_DECREASE_CREDIT: i32 = 200;

#[derive(Debug, Clone)]
pub enum RankStatusInServer {
    NotSubmitted    = -1,
    Pending         = 0,
    Outdated        = 1,
    Ranked          = 2,
    Approved        = 3,
    Qualified       = 4,
    Loved           = 5,
    Unknown,
}

impl RankStatusInServer {
    #[inline(always)]
    pub fn from_api_rank_status(i: i32) -> Self {
        match i {
            -2 => Self::Pending,
            -1 => Self::Pending,
            0 => Self::Pending,
            1 => Self::Ranked,
            2 => Self::Approved,
            3 => Self::Qualified,
            4 => Self::Loved,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubmissionStatus {
    Failed          = 0,
    Passed          = 1,
    PassedAndTop    = 2,
}

impl SubmissionStatus {
    #[inline(always)]
    pub fn val(&self) -> i16 {
        *self as i16
    }
}

#[derive(Debug, Clone)]
pub enum RankStatusInOsuApi {
    Graveyard   = -2,
    Wip         = -1,
    Pending     = 0,
    Ranked      = 1,
    Approved    = 2,
    Qualified   = 3,
    Loved       = 4,
    Unknown,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum ScoreboardType {
    Local   = 0,
    Global  = 1,
    PlayMod = 2,
    Friends = 3,
    Country = 4,
}

impl<'de> Deserialize<'de> for ScoreboardType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let scoreboard_value: u8 = Deserialize::deserialize(deserializer)?;
        match ScoreboardType::parse(scoreboard_value) {
            Some(s) => Ok(s),
            None => Err("invalid scoreboard type value").map_err(D::Error::custom),
        }
    }
}

impl ScoreboardType {
    pub fn parse(value: u8) -> Option<Self> {
        ScoreboardType::from_u8(value)
    }
}
