use actix_web::web::Bytes;

use crate::types::{ClientHashes, ClientInfo, Password, Username};

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
///      2: location (unused1)
///      3: client hash set
///      4: block non-friend pm (unused2)
/// ```
async fn parse_client_info(data_lines: String) -> Result<Vec<String>, ()> {
    let client_info_line: Vec<String> = data_lines.split("|").map(|s| s.to_string()).collect();
    match client_info_line.len() >= 5 {
        true => Ok(client_info_line),
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
async fn parse_client_hashes(client_hashes: String) -> Result<Vec<String>, ()> {
    let hashes_data: Vec<String> = client_hashes
        .split(":")
        .filter(|i| i != &"")
        .map(|s| s.to_string())
        .collect();
    match hashes_data.len() >= 5 {
        true => Ok(hashes_data),
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
    if username.len() < 1 || password.len() < 30 {
        error!(
            "Failed: invalid username or password; username: {}, password: {}",
            username, password
        );
        return Err(-5);
    }
    // Parse client info
    let client_info_line = match parse_client_info(data_lines[2].to_string()).await {
        Ok(client_info_line) => client_info_line,
        Err(_) => {
            error!("Failed: parse_client_info; Request body: @{}@", body);
            return Err(-3);
        }
    };
    // Parse client hashes
    let client_hash_set = match parse_client_hashes(client_info_line[3].to_string()).await {
        Ok(client_hash_set) => client_hash_set,
        Err(_) => {
            error!("Failed: parse_client_hashes; Request body: @{}@", body);
            return Err(-4);
        }
    };

    Ok((username, password, client_info_line, client_hash_set))
}
