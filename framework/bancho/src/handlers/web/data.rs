use super::depends::*;

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
