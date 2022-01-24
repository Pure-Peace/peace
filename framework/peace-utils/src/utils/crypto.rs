use simple_rijndael::{impls::RijndaelCbc, paddings::ZeroPadding, Errors};
use std::{string::FromUtf8Error, time::Instant};

#[derive(Debug)]
pub enum SubmitModularErrors {
    AesDecryptError(Errors),
    StringParseError(FromUtf8Error),
}

#[inline(always)]
/// Decrypt osu!score data with Rijndael-256-cbc algorithm
pub fn submit_modular_decrypt(
    osu_version: i32,
    iv: Vec<u8>,
    score: Vec<u8>,
) -> Result<Vec<String>, SubmitModularErrors> {
    debug!("[SubmitModular] Rijndael-256-cbc decrypt start");
    let start = Instant::now();

    let key = format!("osu!-scoreburgr---------{}", osu_version);
    let rijndael = RijndaelCbc::<ZeroPadding>::new(key.as_bytes(), 32)
        .map_err(|err| SubmitModularErrors::AesDecryptError(err))?;

    let decrypted = rijndael
        .decrypt(&iv, score)
        .map_err(|err| SubmitModularErrors::AesDecryptError(err))?;

    let result = String::from_utf8(decrypted)
        .map_err(|err| SubmitModularErrors::StringParseError(err))?
        .split(':')
        .map(|s| s.into())
        .collect();

    let end = start.elapsed();
    debug!(
        "[SubmitModular] Rijndael-256-cbc decrypt success, time spent: {:?}",
        end
    );
    return Ok(result);
}
