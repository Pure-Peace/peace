use bancho_packets::{self, PacketReader};
use criterion::{criterion_group, criterion_main, Criterion};

fn packets_write_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("packets_write");
    group.bench_function("match_join_failed packet", |b| {
        b.iter(|| bancho_packets::match_join_fail())
    });
    group.bench_function("match_join_failed packet - 2", |b| {
        b.iter(|| {
            bancho_packets::PacketBuilder::new()
                .add(bancho_packets::match_join_fail())
                .write_out()
        })
    });
    group.bench_function("notification packet", |b| {
        b.iter(|| bancho_packets::notification("hello"))
    });
    group.bench_function("login_reply packet", |b| {
        b.iter(|| bancho_packets::login_reply(bancho_packets::LoginFailed::InvalidCredentials))
    });
    group.bench_function("send massage packet", |b| {
        b.iter(|| bancho_packets::send_message("PurePeace", 1001, "May you have enough happiness to make you sweet,enough trials to make you strong,enough sorrow to keep you human,enough hope to make you happy? Always put yourself in others’shoes.If you feel that it hurts you,it probably hurts the other person, too. The happiest of people don’t necessarily have the best of everything;they just make the most of everything that comes along their way.Happiness lies for those who cry,those who hurt, those who have searched,and those who have tried,for only they can appreciate the importance of people. Please send this message to those people who mean something to you,to those who have touched your life in one way or another,to those who make you smile when you really need it,to those that make you see the brighter side of things when you are really down,to those who you want to let them know that you appreciate their friendship.And if you don’t, don’t worry,nothing bad will happen to you,you will just miss out on the opportunity to brighten someone’s day with this message.", "osu"))
    });
    group.bench_function("login mutiple packet test1", |b| {
        b.iter(|| {
            bancho_packets::PacketBuilder::new()
                .add(bancho_packets::login_reply(
                    bancho_packets::LoginSuccess::Verified(1009),
                ))
                .add(bancho_packets::protocol_version(19))
                .add(bancho_packets::notification("Welcome to Peace!"))
                .add(bancho_packets::main_menu_icon(
                    "https://i.kafuu.pro/welcome.png|https://www.baidu.com",
                ))
                .add(bancho_packets::silence_end(0))
                .add(bancho_packets::channel_info_end())
                .write_out()
        })
    });
    group.finish();
}

fn packets_read_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("packets_read");
    let packet = vec![
        24, 0, 0, 32, 0, 0, 0, 11, 30, 230, 172, 162, 232, 191, 142, 230, 130, 168, 239, 188, 140,
        233, 171, 152, 232, 180, 181, 231, 154, 132, 230, 146, 146, 230, 179, 188, 231, 137, 185,
        105, 0, 0, 7, 0, 0, 0, 11, 5, 80, 101, 97, 99, 101, 24, 0, 0, 44, 0, 0, 0, 11, 42, 45, 32,
        79, 110, 108, 105, 110, 101, 32, 85, 115, 101, 114, 115, 58, 32, 50, 10, 45, 32, 87, 101,
        108, 99, 111, 109, 101, 32, 116, 111, 32, 111, 115, 117, 33, 75, 97, 102, 117, 117, 126,
        126, 92, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 4, 0, 0, 0, 232, 3, 0, 0, 75, 0, 0, 4, 0,
        0, 0, 19, 0, 0, 0, 71, 0, 0, 4, 0, 0, 0, 39, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3, 0, 0,
        11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 11, 0, 0, 46, 0, 0, 0, 232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 202, 7, 224,
        54, 0, 0, 0, 0, 100, 112, 123, 63, 41, 0, 0, 0, 135, 96, 87, 56, 0, 0, 0, 0, 1, 0, 0, 0, 7,
        1, 89, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 6, 0, 0, 0, 11, 4, 35, 111, 115, 117, 64, 0,
        0, 11, 0, 0, 0, 11, 9, 35, 97, 110, 110, 111, 117, 110, 99, 101, 64, 0, 0, 8, 0, 0, 0, 11,
        6, 35, 97, 100, 109, 105, 110, 65, 0, 0, 27, 0, 0, 0, 11, 4, 35, 111, 115, 117, 11, 17, 75,
        97, 102, 117, 117, 32, 103, 108, 111, 98, 97, 108, 32, 99, 104, 97, 116, 2, 0, 65, 0, 0,
        31, 0, 0, 0, 11, 9, 35, 97, 110, 110, 111, 117, 110, 99, 101, 11, 16, 65, 110, 110, 111,
        117, 110, 99, 101, 32, 99, 104, 97, 110, 110, 101, 108, 2, 0, 65, 0, 0, 27, 0, 0, 0, 11, 6,
        35, 99, 104, 105, 110, 97, 11, 15, 67, 104, 105, 110, 97, 32, 99, 111, 109, 109, 117, 110,
        105, 116, 121, 1, 0, 65, 0, 0, 31, 0, 0, 0, 11, 8, 35, 101, 110, 103, 108, 105, 115, 104,
        11, 17, 69, 110, 103, 108, 105, 115, 104, 32, 99, 111, 109, 109, 117, 110, 105, 116, 121,
        1, 0, 65, 0, 0, 26, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110, 11, 14, 65, 114, 101, 32,
        121, 111, 117, 32, 97, 100, 109, 105, 110, 63, 2, 0, 65, 0, 0, 71, 0, 0, 0, 11, 6, 35, 108,
        111, 98, 98, 121, 11, 59, 84, 104, 105, 115, 32, 105, 115, 32, 116, 104, 101, 32, 108, 111,
        98, 98, 121, 32, 119, 104, 101, 114, 101, 32, 121, 111, 117, 32, 102, 105, 110, 100, 32,
        103, 97, 109, 101, 115, 32, 116, 111, 32, 112, 108, 97, 121, 32, 119, 105, 116, 104, 32,
        111, 116, 104, 101, 114, 115, 33, 1, 0, 65, 0, 0, 69, 0, 0, 0, 11, 7, 35, 114, 97, 110,
        107, 101, 100, 11, 56, 82, 97, 110, 107, 32, 114, 101, 113, 117, 101, 115, 116, 115, 32,
        109, 97, 112, 115, 32, 119, 105, 108, 108, 32, 98, 101, 32, 112, 111, 115, 116, 101, 100,
        32, 104, 101, 114, 101, 33, 32, 40, 73, 102, 32, 105, 116, 115, 32, 114, 97, 110, 107, 101,
        100, 46, 41, 1, 0, 72, 0, 0, 6, 0, 0, 0, 1, 0, 0, 0, 0, 0, 76, 0, 0, 51, 0, 0, 0, 11, 49,
        104, 116, 116, 112, 115, 58, 47, 47, 105, 46, 107, 97, 102, 117, 117, 46, 112, 114, 111,
        47, 119, 101, 108, 99, 111, 109, 101, 46, 112, 110, 103, 124, 104, 116, 116, 112, 115, 58,
        47, 47, 107, 97, 102, 117, 117, 46, 112, 114, 111, 83, 0, 0, 29, 0, 0, 0, 231, 3, 0, 0, 11,
        8, 67, 104, 105, 110, 111, 66, 111, 116, 24, 48, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 83,
        0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0, 16,
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117,
        114, 101, 80, 101, 97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
    ];

    group.bench_function("big packets read", |b| {
        b.iter(|| {
            let mut reader = PacketReader::from_vec(packet.clone());
            while let Some((packet_id, payload)) = reader.next() {
                let _a = packet_id;
                let _b = payload;
            }
        })
    });
    group.finish();
}

criterion_group!(benches, packets_write_benchmark, packets_read_benchmark);
criterion_main!(benches);
