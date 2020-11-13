mod tests;

pub mod utils;
pub use utils::*;

use crate::{constants::packets::*};

pub fn notification(msg: &str) -> Vec<u8> {
    let mut packet = new(id::BANCHO_NOTIFICATION);
    packet.extend(write_string(msg));
    output(packet)
}

pub fn login_reply(reply: LoginReply) -> Vec<u8> {
    let mut packet = new(id::BANCHO_USER_LOGIN_REPLY);
    packet.extend((reply as i32).to_le_bytes().iter());
    output(packet)
}

pub fn match_join_fail() -> Vec<u8> {
    new(id::BANCHO_MATCH_JOIN_FAIL)
}
