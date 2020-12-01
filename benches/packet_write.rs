#![allow(unused_imports)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[path = "../src/packets/mod.rs"]
mod packets;

#[path = "../src/constants/mod.rs"]
mod constants;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("match_join_failed packet", |b| {
        b.iter(|| packets::match_join_fail())
    });
    c.bench_function("match_join_failed packet - 2", |b| {
        b.iter(|| {
            packets::PacketBuilder::new()
                .add(packets::match_join_fail())
                .write_out()
        })
    });
    c.bench_function("notification packet", |b| {
        b.iter(|| packets::notification("hello"))
    });
    c.bench_function("login_reply packet", |b| {
        b.iter(|| packets::login_reply(constants::packets::LoginFailed::InvalidCredentials))
    });
    c.bench_function("send massage packet", |b| {
        b.iter(|| packets::send_message("PurePeace", 1001, "hello", "osu"))
    });
    c.bench_function("login mutiple packet test", |b| {
        b.iter(|| {
            packets::PacketBuilder::new()
                .add(packets::login_reply(
                    constants::packets::LoginSuccess::Verified(1009),
                ))
                .add(packets::protocol_version(19))
                .add(packets::notification("Welcome to Peace!"))
                .add(packets::main_menu_icon(
                    "https://i.kafuu.pro/welcome.png",
                    "https://www.baidu.com",
                ))
                .add(packets::silence_end(0))
                .add(packets::channel_info_end())
                .write_out()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
