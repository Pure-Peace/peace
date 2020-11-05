use actix_web::web::Bytes;
use nano_leb128::ULEB128;

const NOTI: u16 = 24;

pub fn notification(msg: &str) -> Bytes {
    let a = ULEB128::from(msg.len() as u64);
    println!("{:?}", a);
    let mut b = Bytes::from(format!("<Hx24\x0b03{}", msg));
    b
}
/*     return write(
        Packets.CHO_NOTIFICATION,
        (msg, osuTypes.string)
    ) */
