#![allow(unused_variables)]

use bancho_packets::*;

fn main() {
    // Single packet
    let login_reply_from_server = LoginReply::pack(LoginResult::Failed(
        LoginFailedResaon::InvalidCredentials,
    ));
    let serverside_notification = Notification::pack("hello".into());

    // Multiple packets with Builder
    let packets = PacketBuilder::new()
        .add(LoginReply::pack(LoginResult::Success(1000)))
        .add(ProtocolVersion::pack(19))
        .add(Notification::pack("Welcome to osu!".into()))
        .add(MainMenuIcon::pack(
            "https://image.png".into(),
            "https://url.link".into(),
        ))
        .add(SilenceEnd::pack(0))
        .add(ChannelInfoEnd::pack())
        .build();
}
