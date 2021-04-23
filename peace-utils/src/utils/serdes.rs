use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, Local};
use serde::{de, Deserialize, Deserializer};

#[inline(always)]
pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[inline(always)]
pub fn from_str_optional<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer);
    if s.is_err() {
        return Ok(None);
    };
    Ok(match T::from_str(&s.unwrap()) {
        Ok(t) => Some(t),
        Err(_) => None,
    })
}

#[inline(always)]
pub fn from_str_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let de = String::deserialize(deserializer);
    if de.is_err() {
        return Ok(false);
    };
    match de.unwrap().as_str() {
        "1" => Ok(true),
        _ => Ok(false),
    }
}

#[inline(always)]
pub fn noew_time_local() -> DateTime<Local> {
    Local::now()
}

#[inline(always)]
pub fn try_parse<T>(string: &str) -> Option<T>
where
    T: FromStr,
{
    match T::from_str(string) {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}

pub mod serde_time {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Option<DateTime<Local>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.unwrap().format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer);
        if s.is_err() {
            return Ok(None);
        };
        match Local.datetime_from_str(&s.unwrap(), FORMAT) {
            Ok(t) => Ok(Some(t)),
            Err(_err) => Ok(None),
        }
    }
}
