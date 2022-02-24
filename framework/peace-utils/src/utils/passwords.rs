use argon2::{ThreadMode, Variant, Version};
use rand::Rng;
use std::time::Instant;

lazy_static::lazy_static! {
    static ref ARGON2_CONFIG: argon2::Config<'static> = argon2::Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: 4096,
        time_cost: 3,
        lanes: 1,
        thread_mode: ThreadMode::Sequential,
        secret: &[],
        ad: &[],
        hash_length: 32
    };
}

#[inline]
pub fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

#[inline]
/// Argon2 verify
pub async fn argon2_verify(password_crypted: &str, password: &str) -> bool {
    let argon2_verify_start = Instant::now();
    let verify_result = argon2::verify_encoded(password_crypted, password.as_bytes());
    let argon2_verify_end = argon2_verify_start.elapsed();
    debug!(
        "[argon2_verify] Argon2 verify done, time spent: {:.2?};",
        argon2_verify_end
    );
    match verify_result {
        Ok(result) => result,
        Err(err) => {
            error!(
                "[argon2_verify] Failed to verify argon2: {:?}; crypted: {}, password: {}",
                err, password_crypted, password
            );
            false
        }
    }
}

#[inline]
/// Argon2 encode
pub async fn argon2_encode(password: &[u8]) -> String {
    argon2::hash_encoded(password, rand_string(32).as_bytes(), &ARGON2_CONFIG).unwrap()
}
