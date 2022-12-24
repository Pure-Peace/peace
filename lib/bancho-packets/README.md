# bancho-packets

osu! Bancho data packet reading and writing utilities.

### Test

```
cargo test
```

### Benchmark

```
cargo bench
```

## Reading from osu

```rust
use bancho_packets::{PacketReader, PayloadReader};

// Data packets from osu!
let data = &[
    4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108,
    111, 44, 32, 87, 111, 114, 108, 100, 33, 240, 159, 146, 150, 4, 0, 0,
    0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0, 11, 16, 229, 147, 136, 229, 147,
    136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145, 104, 0, 0, 0, 0,
    0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175, 187, 229, 143, 150, 229,
    174, 140, 228, 186, 134, 239, 188, 129, 239, 188, 129, 226, 156, 168,
];

// Create reader
let mut reader = PacketReader::new(data);

// Read packets
while let Some(packet) = reader.next() {
    print!("{:?}: ", packet.id);
    match packet.payload {
        Some(payload) => {
            // Read payload
            let mut payload_reader = PayloadReader::new(payload);
            println!("{:?}", payload_reader.read::<String>());
        },
        None => println!("Non-payload"),
    }
}
```

### Results

```bash
OSU_PING: Non-payload
BANCHO_NOTIFICATION: Some("Hello, World!💖")
OSU_PING: Non-payload
BANCHO_NOTIFICATION: Some("哈哈【😃】")
BANCHO_ACCOUNT_RESTRICTED: Non-payload
BANCHO_NOTIFICATION: Some("读取完了！！✨")
```

## Writing to osu

```rust
use bancho_packets::*;

// Single packet
let login_reply_from_server = server::login_reply(LoginResult::Failed(
    LoginFailedResaon::InvalidCredentials,
));
let serverside_notification = server::notification("hello");

// Multiple packets with Builder
let packets = PacketBuilder::new()
    .add(&server::login_reply(LoginResult::Success(1000)))
    .add(&server::protocol_version(19))
    .add(&server::notification("Welcome to osu!"))
    .add(&server::main_menu_icon("https://image.png", "https://url.link"))
    .add(&server::silence_end(0))
    .add(&server::channel_info_end())
    .build();

```

## Raw (Build your own packet)

```rust
use bancho_packets::*;

// Build simple packet
let number_data: i32 = 1;
let packet = packet!(PacketId::BANCHO_MATCH_PLAYER_SKIPPED, number_data)

// Complex
pub fn user_stats(
    user_id: i32,
    action: u8,
    info: &str,
    beatmap_md5: &str,
    mods: u32,
    mode: u8,
    beatmap_id: i32,
    ranked_score: i64,
    accuracy: f32,
    playcount: i32,
    total_score: i64,
    rank: i32,
    pp: i16,
) -> Vec<u8> {
    packet!(
        PacketId::BANCHO_USER_STATS,
        data!(
            @capacity { 60 }, // optional set capacity
            user_id,
            action,
            info,
            beatmap_md5,
            mods,
            mode,
            beatmap_id,
            ranked_score,
            accuracy / 100f32,
            playcount,
            total_score,
            rank,
            pp
        )
    )
}

```
