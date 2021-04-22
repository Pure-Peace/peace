use criterion::{criterion_group, criterion_main, Criterion};
use peace_packets;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("match_join_failed packet", |b| {
        b.iter(|| peace_packets::match_join_fail())
    });
    c.bench_function("match_join_failed packet - 2", |b| {
        b.iter(|| {
            peace_packets::PacketBuilder::new()
                .add(peace_packets::match_join_fail())
                .write_out()
        })
    });
    c.bench_function("notification packet", |b| {
        b.iter(|| peace_packets::notification("hello"))
    });
    c.bench_function("login_reply packet", |b| {
        b.iter(|| peace_packets::login_reply(peace_constants::LoginFailed::InvalidCredentials))
    });
    /* c.bench_function("send massage packet", |b| {
        b.iter(|| peace_packets::send_message("PurePeace", 1001, "hello", "osu"))
    }); */
    c.bench_function("login mutiple packet test1", |b| {
        b.iter(|| {
            peace_packets::PacketBuilder::new()
                .add(peace_packets::login_reply(
                    peace_constants::LoginSuccess::Verified(1009),
                ))
                .add(peace_packets::protocol_version(19))
                .add(peace_packets::notification("Welcome to Peace!"))
                .add(peace_packets::main_menu_icon(
                    "https://i.kafuu.pro/welcome.png|https://www.baidu.com",
                ))
                .add(peace_packets::silence_end(0))
                .add(peace_packets::channel_info_end())
                .write_out()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
