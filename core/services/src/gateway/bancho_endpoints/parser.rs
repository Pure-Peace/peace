use super::ParseLoginDataError;
use peace_pb::bancho::{ClientHashes, LoginRequest};

pub fn parse_osu_login_request_body(
    body: Vec<u8>,
) -> Result<LoginRequest, ParseLoginDataError> {
    #[inline]
    fn shift<T>(vec: &mut Vec<T>) -> T {
        vec.remove(0)
    }

    let body = String::from_utf8(body)
        .map_err(ParseLoginDataError::InvalidRequestBody)?;

    let mut lines = tools::split_string(&body, '\n');

    if lines.len() < 3 {
        return Err(ParseLoginDataError::InvalidLoginData);
    }

    let username = shift(&mut lines);
    let password = shift(&mut lines);

    if username.is_empty() || password.len() != 32 {
        return Err(ParseLoginDataError::InvalidUserInfo);
    }

    let mut client_info = tools::split_string(&shift(&mut lines), '|');

    if client_info.len() < 5 {
        return Err(ParseLoginDataError::InvalidClientInfo);
    }

    let client_version = shift(&mut client_info);

    // Parse utc offset
    let utc_offset = shift(&mut client_info).parse::<i32>().unwrap_or(0);

    // Display city in bancho or not
    let display_city = shift(&mut client_info) == "1";

    // Client hashes
    let mut client_hashes = tools::split_string(&shift(&mut client_info), ':');

    if client_hashes.len() < 5 {
        return Err(ParseLoginDataError::InvalidClientHashes);
    }

    // Only allow friend's pm
    let only_friend_pm_allowed = shift(&mut client_info) == "1";

    let path_hash = shift(&mut client_hashes);
    let adapters = shift(&mut client_hashes);
    let adapters_hash = shift(&mut client_hashes);
    let uninstall_id = shift(&mut client_hashes);
    let disk_id = shift(&mut client_hashes);

    Ok(LoginRequest {
        username,
        password,
        client_version,
        utc_offset,
        display_city,
        only_friend_pm_allowed,
        client_hashes: Some(ClientHashes {
            path_hash,
            adapters,
            adapters_hash,
            uninstall_id,
            disk_id,
        }),
    })
}
