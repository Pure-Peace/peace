use simple_rijndael::{impls::RijndaelCbc, paddings::ZeroPadding};
use std::time::Instant;

#[inline(always)]
/// Decrypt osu!score data with Rijndael-256-cbc algorithm
pub fn submit_modular_decrypt(
    osu_version: i32,
    iv: Vec<u8>,
    score: Vec<u8>,
) -> Result<Vec<String>, &'static str> {
    debug!("[SubmitModular] Rijndael-256-cbc decrypt start");
    let start = Instant::now();

    let key = format!("osu!-scoreburgr---------{}", osu_version)
        .as_bytes()
        .into();
    let rijndael = RijndaelCbc::new(key, 32)?;
    let decryp_result = String::from_utf8(rijndael.decrypt(iv, score)?)
        .map_err(|_| "String parse error")?
        .split(":")
        .map(|s| s.into())
        .collect();

    let end = start.elapsed();
    debug!(
        "[SubmitModular] Rijndael-256-cbc decrypt success, time spent: {:?}",
        end
    );
    return Ok(decryp_result);
}
