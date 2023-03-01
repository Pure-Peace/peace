use super::Error;
use peace_pb::bancho_rpc::{ClientHashes, LoginRequest};

pub fn parse_osu_login_request_body(
    body: Vec<u8>,
) -> Result<LoginRequest, Error> {
    let body = String::from_utf8(body)
        .map_err(|_| Error::Login("invalid request body".into()))?;

    let mut lines = tools::split_string(&body, '\n');

    if lines.len() < 3 {
        return Err(Error::Login("invalid data".into()))
    }

    let username = std::mem::take(&mut lines[0]);
    let password = std::mem::take(&mut lines[1]);

    if username.is_empty() || password.len() != 32 {
        return Err(Error::Login("invalid user info".into()))
    }

    let mut client_info = tools::split_string(&lines[2], '|');

    if client_info.len() < 5 {
        return Err(Error::Login("invalid client info".into()))
    }

    let client_version = std::mem::take(&mut client_info[0]);

    // Parse utc offset
    let utc_offset = client_info[1].parse::<i32>().unwrap_or(0);

    // Display city in bancho or not
    let display_city = client_info[2].as_str() == "1";

    // Client hashes
    let mut client_hashes = tools::split_string(&client_info[3], ':');

    if client_hashes.len() < 5 {
        return Err(Error::Login("invalid client hashes".into()))
    }

    // Only allow friend's pm
    let only_friend_pm_allowed = client_info[4].as_str() == "1";

    let path_hash = std::mem::take(&mut client_hashes[0]);
    let adapters = std::mem::take(&mut client_hashes[1]);
    let adapters_hash = std::mem::take(&mut client_hashes[2]);
    let uninstall_id = std::mem::take(&mut client_hashes[3]);
    let disk_id = std::mem::take(&mut client_hashes[4]);

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
