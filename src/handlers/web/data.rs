use super::depends::*;

/// Query Data
///
/// GET /web/lastfm.php
///
/// ```
/// Lastfm {
///     b: String = beatmap ban,
///     action: String,
///     us: String = username,
///     ha: String = password hash,
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct Lastfm {
    pub b: String,
    pub action: String,
    pub us: String,
    pub ha: String,
}

/// Query Data
///
/// GET /web/check-updates.php
///
/// ```
/// CheckUpdates {
///     action: String = [check, path, error],
///     stream: String = [cuttingedge, stable40, beta40, stable],
///     time: String = timeStamp,
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct CheckUpdates {
    pub action: String,
    pub stream: String,
    pub time: String,
}

/// Query Data
///
/// GET /web/bancho_connect.php
///
/// ```
/// BanchoConnect {
///     v: String = osu version,
///     u: String = username,
///     h: String = password hash,
///     fx: String = donet env,
///     ch: String = hardware hashes,
///     retry: i32 = retries,
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct BanchoConnect {
    pub v: String,
    pub u: String,
    pub h: String,
    pub fx: String,
    pub ch: String,
    pub retry: i32,
}

/// Multipart Form-data
///
/// POST /web/osu-session.php
///
/// ```
/// OsuSession {
///     u: String = username,
///     h: String = password hash,
///     action: String = [check, submit],
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct OsuSession {
    pub u: Option<String>,
    pub h: String,
    pub action: String,
}

/// Multipart Form-data
///
/// POST /web/osu-error.php
///
/// ```
/// OsuError {
///     u: String = username,
///     p: String = password hash,
///     i: i32,
///     osumode: String = [Menu],
///     gamemode: String = [Osu, Taiko, Mania, Catch],
///     gametime: u32,
///     audiotime: u32,
///     culture: String = [zh-CN],
///     beatmap_id: u32,
///     beatmap_checksum: String,
///     exception: String = [System.Exception],
///     feedback: String = [update error],
///     stacktrace: String,
///     soft: String = [True, False],
///     beatmap_count: u32,
///     compatibility: u32,
///     version: String = osu version,
///     exehash: String,
///     config: String = osu config(ini),
/// }
///
/// ```
#[derive(Debug, Deserialize)]
pub struct OsuError {
    pub u: String,
    pub p: String,
    pub i: i32,
    pub osumode: String,
    pub gamemode: String,
    pub gametime: u32,
    pub audiotime: u32,
    pub culture: String,
    pub beatmap_id: u32,
    pub beatmap_checksum: String,
    pub exception: String,
    pub feedback: String,
    pub stacktrace: String,
    pub soft: String,
    pub beatmap_count: u32,
    pub compatibility: u32,
    pub version: String,
    pub exehash: String,
    pub config: String,
}
