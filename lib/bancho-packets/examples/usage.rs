#![allow(unused_variables)]

use bancho_packets::*;

fn main() {
    // Single packet
    let login_reply_from_server = server::LoginReply::pack(
        LoginResult::Failed(LoginFailedReason::InvalidCredentials),
    );
    let serverside_notification = server::Notification::pack("hello".into());

    // Multiple packets with Builder
    let packets = PacketBuilder::new()
        .add(server::LoginReply::pack(LoginResult::Success(1000)))
        .add(server::ProtocolVersion::pack(19))
        .add(server::Notification::pack("Welcome to osu!".into()))
        .add(server::MainMenuIcon::pack(
            "https://image.png".into(),
            "https://url.link".into(),
        ))
        .add(server::SilenceEnd::pack(0))
        .add(server::ChannelInfoEnd::pack())
        .build();
}
