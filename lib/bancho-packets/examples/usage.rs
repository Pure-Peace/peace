use bancho_packets::*;

fn main() {
    // Single packet
    let login_reply_from_server = server::login_reply(LoginResult::Failed(
        LoginFailedResaon::InvalidCredentials,
    ));
    let serverside_notification = server::notification("hello");

    // Multiple packets with Builder
    let packets = PacketBuilder::new()
        .add(server::login_reply(LoginResult::Success(1000)))
        .add(server::protocol_version(19))
        .add(server::notification("Welcome to osu!"))
        .add(server::main_menu_icon("https://image.png", "https://url.link"))
        .add(server::silence_end(0))
        .add(server::channel_info_end())
        .build();
}
