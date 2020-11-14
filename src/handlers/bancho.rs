#![allow(unused_variables)]
#![allow(unused_imports)]

use actix_web::{http::HeaderMap, web::Bytes};

use crate::packets::PacketData;
use crate::{constants, packets};

/// Get Login data lines
/// ```
///  rows:
///      0: username
///      1: password hash
///      2: client info and hardware info
/// ```
fn parse_login_data(body: &String) -> Result<Vec<&str>, ()> {
    let data_lines: Vec<&str> = body.split("\n").filter(|i| i != &"").collect();
    match data_lines.len() < 3 {
        true => Ok(data_lines),
        false => Err(()),
    }
}

/// Get client info lines
/// ```
///  rows:
///      0: osu version
///      1: time offset (utc)
///      2: location (unused1)
///      3: client hash set
///      4: block non-friend pm (unused2)
/// ```
fn parse_client_info(data_lines: Vec<&str>) -> Result<Vec<&str>, ()> {
    let client_info_line: Vec<&str> = data_lines[2].split("|").collect();
    match client_info_line.len() < 5 {
        true => Ok(client_info_line),
        false => Err(()),
    }
}

/// Get client hash set
/// ```
///  rows:
///      0: osu path md5
///      1: adapters (network physical addresses delimited by '.')
///      2: adapters md5
///      3: uniqueid1 (osu! uninstall id)
///      4: uniqueid2 (disk signature/serial num)
/// ```
fn parse_client_hashes(client_hashes: &str) -> Result<Vec<&str>, ()> {
    let hashes_data: Vec<&str> = client_hashes.split(":").filter(|i| i != &"").collect();
    match hashes_data.len() < 5 {
        true => Ok(hashes_data),
        false => Err(()),
    }
}

/// Bancho login handler
pub async fn login(body: &Bytes, request_ip: String, osu_version: String) -> (PacketData, String) {
    // Get string body
    let body = String::from_utf8(body.to_vec()).unwrap();

    // Parse body
    let data_lines = parse_login_data(&body).unwrap();

    // Parse client info
    let client_info_line = parse_client_info(data_lines.clone()).unwrap();

    // Parse client hashes
    let client_hash_set = parse_client_hashes(client_info_line[3]).unwrap();

    // println!("{:?}", data_lines);

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
        "ggg".to_string(),
    )
}
