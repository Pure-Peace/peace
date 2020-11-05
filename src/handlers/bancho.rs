use actix_web::{http::HeaderMap, web::Bytes};

use crate::packets;

/// Bancho login handler
pub async fn login(body: &Bytes, request_ip: String, osu_version: String) -> (Bytes, String) {
    // Get string body
    let body = String::from_utf8(body.to_vec()).unwrap();

    // Data lines
    //  rows: 
    //      0: username
    //      1: password hash
    //      2: client info and hardware info 
    let data_lines: Vec<&str> = body.split("\n").filter(|i| i != &"").collect();
    if data_lines.len() < 3 {
        panic!("gg1");
    }

    // Get client info lines
    //  rows:
    //      0: osu version
    //      1: time offset
    //      2: unused1
    //      3: client hash set
    //      4: unused2
    let client_info_line: Vec<&str> = data_lines[2].split("|").collect();
    if client_info_line.len() < 5 {
        panic!("gg2");
    }

    // Get client hash set
    //  rows:
    //      0: osu path md5
    //      1: adapters (network physical addresses delimited by '.')
    //      2: adapters md5
    //      3: uniqueid1 (osu! uninstall id)
    //      4: uniqueid2 (disk signature/serial num)
    let client_hash_set: Vec<&str> = client_info_line[3].split(":").filter(|i| i != &"").collect();
    if client_info_line.len() < 5 {
        panic!("gg3");
    }



    
    println!("data_lines: {:?}\nclient_info_line: {:?}\nclient_hash_set: {:?}", data_lines, client_info_line, client_hash_set);


    (packets::notification("asd"), "ggg".to_string())
}
