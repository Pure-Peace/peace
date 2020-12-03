use actix_web::web::Bytes;

use crate::{
    constants::{ClientHashes, ClientInfo},
    types::{Password, Username},
};

#[inline(always)]
/// Get Login data lines
/// ```
///  rows:
///      0: username
///      1: password hash
///      2: client info and hardware info
/// ```
async fn parse_data_lines(body: &String) -> Result<Vec<String>, ()> {
    let data_lines: Vec<String> = body
        .split("\n")
        .filter(|i| i != &"")
        .map(|s| s.to_string())
        .collect();
    match data_lines.len() >= 3 {
        true => Ok(data_lines),
        false => Err(()),
    }
}
#[inline(always)]
/// Get client info lines
/// ```
///  rows:
///      0: osu version
///      1: time offset (utc)
///      2: display city (location unused)
///      3: client hash set
///      4: block non-friend pm (unused)
/// ```
async fn parse_client_info(data_lines: String) -> Result<(ClientInfo, String), ()> {
    let client_info_line: Vec<String> = data_lines.split("|").map(|s| s.to_string()).collect();
    match client_info_line.len() >= 5 {
        true => {
            // Parse osu version
            // TODO: check the osu version
            let osu_version = client_info_line[0].clone();

            // Parse utc offset
            let utc_offset = client_info_line[1].parse().unwrap();

            // Display city in bancho or not
            let display_city = client_info_line[2] == "1".to_string();

            // Only allow friend's pm
            let only_friend_pm_allowed = client_info_line[4] == "1".to_string();

            Ok((
                ClientInfo {
                    osu_version,
                    utc_offset,
                    display_city,
                    only_friend_pm_allowed,
                },
                client_info_line[3].clone(),
            ))
        }
        false => Err(()),
    }
}
#[inline(always)]
/// Get client hash set
/// ```
///  rows:
///      0: osu path md5
///      1: adapters (network physical addresses delimited by '.')
///      2: adapters md5
///      3: uniqueid1 (osu! uninstall id)
///      4: uniqueid2 (disk signature/serial num)
/// ```
async fn parse_client_hashes(client_hashes: String) -> Result<ClientHashes, ()> {
    let hashes_data: Vec<String> = client_hashes
        .split(":")
        .filter(|i| i != &"")
        .map(|s| s.to_string())
        .collect();
    match hashes_data.len() >= 5 {
        true => Ok(ClientHashes {
            osu_path: hashes_data[0].clone(),
            adapters: hashes_data[1].clone(),
            adapters_hash: hashes_data[2].clone(),
            uninstall_id: hashes_data[3].clone(),
            disk_id: hashes_data[4].clone(),
        }),
        false => Err(()),
    }
}

#[inline(always)]
/// Parse login data
pub async fn parse_login_data(
    body: &Bytes,
) -> Result<(Username, Password, ClientInfo, ClientHashes), i32> {
    // Body to string
    let body = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(_) => {
            error!("Failed: parse_body;");
            return Err(-1);
        }
    };
    // Parse body
    let data_lines = match parse_data_lines(&body).await {
        Ok(data_lines) => data_lines,
        Err(_) => {
            error!("Failed: parse_data_lines; Request body: @{}@", body);
            return Err(-2);
        }
    };
    // Check username and password
    let username = data_lines[0].clone();
    let password = data_lines[1].clone();
    // Password (md5) length is 32
    if username.len() < 1 || password.len() != 32 {
        error!(
            "Failed: invalid username or password; username: {}, password: {}",
            username, password
        );
        return Err(-5);
    }
    // Parse client info
    let (client_info, client_hash_set) = match parse_client_info(data_lines[2].to_string()).await {
        Ok(result) => result,
        Err(_) => {
            error!("Failed: parse_client_info; Request body: @{}@", body);
            return Err(-3);
        }
    };
    // Parse client hashes
    let client_hashes = match parse_client_hashes(client_hash_set).await {
        Ok(client_hashes) => client_hashes,
        Err(_) => {
            error!("Failed: parse_client_hashes; Request body: @{}@", body);
            return Err(-4);
        }
    };

    Ok((username, password, client_info, client_hashes))
}
