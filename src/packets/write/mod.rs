mod tests;

pub mod utils;
pub use utils::*;

use crate::constants::packets::*;

/// #5: BANCHO_USER_LOGIN_REPLY
pub fn login_reply(reply: LoginReply) -> Vec<u8> {
    PacketBuilder::id(id::BANCHO_USER_LOGIN_REPLY)
        .add(write_integer(reply as i32))
        .write_out()
}

/// #7: BANCHO_SEND_MESSAGE
pub fn send_message(sender: &str, sender_id: i32, content: &str, channel: &str) -> Vec<u8> {
    PacketBuilder::id(id::BANCHO_SEND_MESSAGE)
        .add(write_message(sender, sender_id, content, channel))
        .write_out()
}

/// #8: BANCHO_PONG
pub fn pong() -> Vec<u8> {
    new(id::BANCHO_PONG)
}

/// #9: BANCHO_SEND_MESSAGE
pub fn change_username(username_old: &str, username_new: &str) -> Vec<u8> {
    PacketBuilder::id(id::BANCHO_HANDLE_IRC_CHANGE_USERNAME)
        .add(write_string(&format!(
            "{}>>>>{}",
            username_old, username_new
        )))
        .write_out()
}

/// #24: BANCHO_NOTIFICATION
pub fn notification(msg: &str) -> Vec<u8> {
    PacketBuilder::id(id::BANCHO_NOTIFICATION)
        .add(write_string(msg))
        .write_out()
}

/// #37: BANCHO_MATCH_JOIN_FAIL
pub fn match_join_fail() -> Vec<u8> {
    new(id::BANCHO_MATCH_JOIN_FAIL)
}
