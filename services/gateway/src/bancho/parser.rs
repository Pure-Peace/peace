use axum::response::Response;
use peace_api::error::{map_err, Error};
use peace_pb::services::bancho::{ClientHashes, LoginRequest};

pub fn parse_osu_login_data_lines(
    body: Vec<u8>,
) -> Result<Vec<String>, Response> {
    let body = String::from_utf8(body).map_err(map_err)?;

    let lines = body
        .split('\n')
        .filter(|i| i != &"")
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    if lines.len() < 3 {
        return Err(Error::Anyhow(anyhow!("Invalid login data.")).into());
    }

    Ok(lines)
}

pub fn parse_osu_login_request_data(
    mut lines: Vec<String>,
) -> Result<LoginRequest, Response> {
    let username = std::mem::take(&mut lines[0]);
    let password = std::mem::take(&mut lines[1]);

    if username.len() < 2 || password.len() != 32 {
        return Err(
            Error::Anyhow(anyhow!("Invalid username or password.")).into()
        );
    }

    let mut client_info =
        lines[2].split('|').map(|s| s.to_owned()).collect::<Vec<String>>();

    if client_info.len() < 5 {
        return Err(Error::Anyhow(anyhow!("Invalid client info.")).into());
    }

    let client_version = std::mem::take(&mut client_info[0]);

    // Parse utc offset
    let utc_offset = client_info[1].parse::<i32>().map_err(map_err)?;

    // Display city in bancho or not
    let display_city = client_info[2].as_str() == "1";

    // Client hashes
    let mut client_hashes = std::mem::take(&mut client_info[3])
        .split(':')
        .filter(|i| i != &"")
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    if client_hashes.len() < 5 {
        return Err(Error::Anyhow(anyhow!("Invalid client hashes.")).into());
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
        client_hashes: Some(ClientHashes {
            path_hash,
            adapters,
            adapters_hash,
            uninstall_id,
            disk_id,
        }),
        utc_offset,
        display_city,
        only_friend_pm_allowed,
    })
}
