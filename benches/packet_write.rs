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
    c.bench_function("notification packet", |b| {
        b.iter(|| packets::notification("hello"))
    });
    c.bench_function("login_reply packet", |b| {
        b.iter(|| packets::login_reply(constants::packets::LoginReply::InvalidCredentials))
    });
    c.bench_function("send massage packet", |b| {
        b.iter(|| packets::send_message("PurePeace", 1001, "hello", "osu"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
