# bancho-packets

Bancho packet reading and writing utilities.

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
let data = vec![
    4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108, 111, 44, 32, 87,
    111, 114, 108, 100, 33, 240, 159, 146, 150, 4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0,
    11, 16, 229, 147, 136, 229, 147, 136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145,
    104, 0, 0, 0, 0, 0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175, 187, 229, 143, 150,
    229, 174, 140, 228, 186, 134, 239, 188, 129, 239, 188, 129, 226, 156, 168,
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
use bancho_packets::{LoginFailed, PacketBuilder, server_packet};

// Single packet
let data = server_packet::login_reply(LoginFailed::InvalidCredentials);
let data1 = server_packet::notification("hello");

// Multiple packets with Builder
let data3 = PacketBuilder::new()
    .add(server_packet::login_reply(
        server_packet::LoginSuccess::Verified(1009),
    ))
    .add(server_packet::protocol_version(19))
    .add(server_packet::notification("Welcome to Peace!"))
    .add(server_packet::main_menu_icon(
        "https://xxx.png|https://example.com",
    ))
    .add(server_packet::silence_end(0))
    .add(server_packet::channel_info_end())
    .write_out();

```

## Raw (Build your own packet)

```rust
use bancho_packets::prelude::*;

// Build simple packet
let number_data: i32 = 1;
let packet = build!(PacketId::BANCHO_MATCH_PLAYER_SKIPPED, number_data)

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
    build!(
        PacketId::BANCHO_USER_STATS,
        data!(
            { 60 }; // initial data size
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
