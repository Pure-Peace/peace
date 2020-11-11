pub mod id;

pub fn notification(msg: &str) -> Vec<u8> {
    let mut packet = new(id::BANCHO_NOTIFICATION);
    packet.extend(write_string(msg));
    output(packet)
}

pub fn match_join_fail() -> Vec<u8> {
    output(new(id::BANCHO_MATCH_JOIN_FAIL))
}

/// Create a empty packets
pub fn empty() -> Vec<u8> {
    Vec::with_capacity(11)
}

/// Initial a packets by id
///
/// Packets posit:
/// ```
/// [0..=2]: packet id
/// [3..=6]: packet length
/// [7..=N]: data length(uleb128)  + data
/// ```
pub fn new(packet_id: u8) -> Vec<u8> {
    vec![packet_id, 0, 0, 0, 0, 0, 0]
}

/// Add packet length
pub fn output(mut packet: Vec<u8>) -> Vec<u8> {
    for (index, value) in ((packet.len() - 7) as i32).to_le_bytes().iter().enumerate() {
        packet[3 + index] = *value;
    }
    packet
}

/// Write string packet
pub fn write_string(string: &str) -> Vec<u8> {
    let byte_length = string.len();
    let mut data: Vec<u8> = Vec::with_capacity(byte_length + 3);
    if byte_length > 0 {
        data.push(11); // 0x0b, means not empty
        data.extend(uleb128(byte_length as u32));
        data.extend(string.as_bytes());
    } else {
        data.push(0); // 0x00, means empty
    }
    data
}

/// Unsigned to uleb128
fn uleb128(mut unsigned: u32) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(2);
    while unsigned >= 0x80 {
        data.push(((unsigned & 0x7f) | 0x80) as u8);
        unsigned >>= 7;
    }
    data.push(unsigned as u8);
    data
}
