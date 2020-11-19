#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::web::{Bytes, Data};
use actix_web::{http::HeaderMap, HttpRequest};

use crate::{constants, packets};
use crate::{
    objects::{Player, PlayerSessions},
    packets::PacketData,
};

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
async fn parse_login_data(
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
    if username.len() < 1 || password.len() < 33 {
        error!("Failed: invalid username or password; username: {}, password: {}", username, password);
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

#[inline(always)]
/// Bancho login handler
pub async fn login(
    req: HttpRequest,
    body: &Bytes,
    request_ip: String,
    osu_version: String,
    player_sessions: Data<PlayerSessions>,
) -> (PacketData, String) {
    let parse_start = std::time::Instant::now();
    // Parse login data first
    let (username, password, client_info_line, client_hash_set) = match parse_login_data(body).await
    {
        Ok(login_data) => login_data,
        Err(err_integer) => {
            error!(
                "Failed: parse_login_data; request_ip: {}; osu_version: {}",
                request_ip, osu_version
            );
            return (vec![0], "login failed".to_string());
        }
    };
    let parse_duration = parse_start.elapsed();
    debug!(
        "Login - data parsed, time spent: {:.2?}; ip: {}, osu_version: {};",
        parse_duration, request_ip, osu_version
    );

    let player = Player {
        id: 1,
        name: "world".to_string(),
        money: 10000,
        age: 16,
    };

    let token = player_sessions.login(player).await;
    /* println!(
        "created a player: {}\nnow sessions:  {:?}",
        token,
        player_sessions.map_to_string().await
    ); */

    /* let packet = packets::PacketBuilder::new()
    .add(packets::notification("hihi"))
    //.add(packets::login_reply(constants::packets::LoginReply::AccountPasswordRest))
    .add(packets::notification("you' re fired"))
    .add(packets::rtx("you' re fired"))
    .add(packets::bancho_restart(3000))
    .done(); */
    //println!("data_lines: {:?}\nclient_info_line: {:?}\nclient_hash_set: {:?}", data_lines, client_info_line, client_hash_set);

    (
        vec![
            24, 0, 0, 32, 0, 0, 0, 11, 30, 230, 172, 162, 232, 191, 142, 230, 130, 168, 239, 188,
            140, 233, 171, 152, 232, 180, 181, 231, 154, 132, 230, 146, 146, 230, 179, 188, 231,
            137, 185, 105, 0, 0, 7, 0, 0, 0, 11, 5, 80, 101, 97, 99, 101, 24, 0, 0, 44, 0, 0, 0,
            11, 42, 45, 32, 79, 110, 108, 105, 110, 101, 32, 85, 115, 101, 114, 115, 58, 32, 50,
            10, 45, 32, 87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 111, 115, 117, 33, 75,
            97, 102, 117, 117, 126, 126, 92, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 4, 0, 0, 0,
            232, 3, 0, 0, 75, 0, 0, 4, 0, 0, 0, 19, 0, 0, 0, 71, 0, 0, 4, 0, 0, 0, 39, 0, 0, 0, 83,
            0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0,
            16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 0, 46, 0, 0, 0, 232, 3, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 202, 7, 224, 54, 0, 0, 0, 0, 100, 112, 123, 63, 41, 0, 0, 0,
            135, 96, 87, 56, 0, 0, 0, 0, 1, 0, 0, 0, 7, 1, 89, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 64, 0,
            0, 6, 0, 0, 0, 11, 4, 35, 111, 115, 117, 64, 0, 0, 11, 0, 0, 0, 11, 9, 35, 97, 110,
            110, 111, 117, 110, 99, 101, 64, 0, 0, 8, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110,
            65, 0, 0, 27, 0, 0, 0, 11, 4, 35, 111, 115, 117, 11, 17, 75, 97, 102, 117, 117, 32,
            103, 108, 111, 98, 97, 108, 32, 99, 104, 97, 116, 2, 0, 65, 0, 0, 31, 0, 0, 0, 11, 9,
            35, 97, 110, 110, 111, 117, 110, 99, 101, 11, 16, 65, 110, 110, 111, 117, 110, 99, 101,
            32, 99, 104, 97, 110, 110, 101, 108, 2, 0, 65, 0, 0, 27, 0, 0, 0, 11, 6, 35, 99, 104,
            105, 110, 97, 11, 15, 67, 104, 105, 110, 97, 32, 99, 111, 109, 109, 117, 110, 105, 116,
            121, 1, 0, 65, 0, 0, 31, 0, 0, 0, 11, 8, 35, 101, 110, 103, 108, 105, 115, 104, 11, 17,
            69, 110, 103, 108, 105, 115, 104, 32, 99, 111, 109, 109, 117, 110, 105, 116, 121, 1, 0,
            65, 0, 0, 26, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110, 11, 14, 65, 114, 101, 32,
            121, 111, 117, 32, 97, 100, 109, 105, 110, 63, 2, 0, 65, 0, 0, 71, 0, 0, 0, 11, 6, 35,
            108, 111, 98, 98, 121, 11, 59, 84, 104, 105, 115, 32, 105, 115, 32, 116, 104, 101, 32,
            108, 111, 98, 98, 121, 32, 119, 104, 101, 114, 101, 32, 121, 111, 117, 32, 102, 105,
            110, 100, 32, 103, 97, 109, 101, 115, 32, 116, 111, 32, 112, 108, 97, 121, 32, 119,
            105, 116, 104, 32, 111, 116, 104, 101, 114, 115, 33, 1, 0, 65, 0, 0, 69, 0, 0, 0, 11,
            7, 35, 114, 97, 110, 107, 101, 100, 11, 56, 82, 97, 110, 107, 32, 114, 101, 113, 117,
            101, 115, 116, 115, 32, 109, 97, 112, 115, 32, 119, 105, 108, 108, 32, 98, 101, 32,
            112, 111, 115, 116, 101, 100, 32, 104, 101, 114, 101, 33, 32, 40, 73, 102, 32, 105,
            116, 115, 32, 114, 97, 110, 107, 101, 100, 46, 41, 1, 0, 72, 0, 0, 6, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 76, 0, 0, 51, 0, 0, 0, 11, 49, 104, 116, 116, 112, 115, 58, 47, 47, 105, 46,
            107, 97, 102, 117, 117, 46, 112, 114, 111, 47, 119, 101, 108, 99, 111, 109, 101, 46,
            112, 110, 103, 124, 104, 116, 116, 112, 115, 58, 47, 47, 107, 97, 102, 117, 117, 46,
            112, 114, 111, 83, 0, 0, 29, 0, 0, 0, 231, 3, 0, 0, 11, 8, 67, 104, 105, 110, 111, 66,
            111, 116, 24, 48, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3,
            0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101,
            97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
        ],
        token,
    )
}
