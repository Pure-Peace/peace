use actix_web::web::Bytes;

pub mod id;

pub async fn notification(msg: &str) -> Vec<u8> {
    let mut ret: Vec<u8> = vec![id::BANCHO_NOTIFICATION, 0, 0];
    ret.extend(write_string(msg).await);
    ret.splice(3..3, vec![(ret.len() - 3) as u8, 0, 0, 0]);
    ret
}
/*     return write(
    Packets.CHO_NOTIFICATION,
    (msg, osuTypes.string)
) */

pub async fn write_packet(packet: u8) {
    let mut ret: Vec<u8> = vec![packet, 0, 0];
    ret.extend(write_string("阿萨德").await);
    ret.splice(3..3, vec![(ret.len() - 3) as u8, 0, 0, 0]);

    //assert_eq!(ret, b"\x18\x00\x00\x03\x00\x00\x00\x0b\x01a".to_vec());
    println!("{:?}", ret);
}

pub async fn write_string(s: &str) -> Vec<u8> {
    let byte_data = s.as_bytes();
    let byte_length = byte_data.len();
    // [0] is b"\x00", empty
    let mut data: Vec<u8> = vec![0];
    if byte_length > 0 {
        // [11] is b"\x0b", not empty
        let mut content = vec![11];
        content.extend(write_uleb128(byte_length).await);
        content.extend(byte_data);
        data.splice(0..1, content);
    }
    data
}

pub async fn write_uleb128(mut num: usize) -> Vec<u8> {
    let mut out: Vec<u8> = vec![];
    while num >= 0x80 {
        out.push(((num & 0x7f) | 0x80) as u8);
        num >>= 7;
    }
    out.push(num as u8);
    out
}
