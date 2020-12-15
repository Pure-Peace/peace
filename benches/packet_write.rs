#![allow(unused_imports)]
#![allow(dead_code)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[macro_use]
extern crate log;
extern crate config;
extern crate serde;

#[path = "../src/constants/mod.rs"]
mod constants;
#[path = "../src/database/mod.rs"]
mod database;
#[path = "../src/events/mod.rs"]
mod events;
#[path = "../src/handlers/mod.rs"]
mod handlers;
#[path = "../src/objects/mod.rs"]
mod objects;
#[path = "../src/packets/mod.rs"]
mod packets;
#[path = "../src/renders/mod.rs"]
mod renders;
#[path = "../src/routes/mod.rs"]
mod routes;
#[path = "../src/settings/mod.rs"]
mod settings;
#[path = "../src/types/mod.rs"]
mod types;
#[path = "../src/utils/mod.rs"]
mod utils;

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
        b.iter(|| packets::login_reply(constants::LoginFailed::InvalidCredentials))
    });
    /* c.bench_function("send massage packet", |b| {
        b.iter(|| packets::send_message("PurePeace", 1001, "hello", "osu"))
    }); */
    c.bench_function("login mutiple packet test1", |b| {
        b.iter(|| {
            packets::PacketBuilder::new()
                .add(packets::login_reply(constants::LoginSuccess::Verified(
                    1009,
                )))
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
