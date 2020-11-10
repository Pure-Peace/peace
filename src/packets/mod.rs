use actix_web::web::Bytes;

pub mod id;

/// Create a empty packets
pub fn empty() -> Vec<u8> {
    Vec::with_capacity(7)
}

/// Initial a packets by id
pub fn new(packet_id: u8) -> Vec<u8> {
    vec![packet_id, 0, 0]
}

pub fn output(mut ret: Vec<u8>) -> Vec<u8> {
    ret.splice(3..3, vec![(ret.len() - 3) as u8, 0, 0, 0]);
    ret
}

pub fn notification(msg: &str) -> Vec<u8> {
    let mut ret = new(id::BANCHO_NOTIFICATION);
    ret.extend(write_string(msg));
    output(ret)
}

pub fn write_string(string: &str) -> Vec<u8> {
    let byte_data = string.as_bytes();
    let byte_length = string.len();
    let mut data: Vec<u8> = Vec::with_capacity(byte_length + 5);
    if byte_length > 0 {
        data.push(11); // 0x0b, means not empty
        data.extend(uleb128(byte_length as u32));
        data.extend(byte_data);
    } else {
        data.push(0); // 0x00, means empty
    }
    data
}

/// Unsigned to uleb128
fn uleb128(mut unsigned: u32) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::new();
    while unsigned >= 0x80 {
        data.push(((unsigned & 0x7f) | 0x80) as u8);
        unsigned >>= 7;
    }
    data.push(unsigned as u8);
    data
}
