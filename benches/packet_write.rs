use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[path = "../src/packets/mod.rs"]
mod packets;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("match_join_failed packet", |b| {
        b.iter(|| packets::match_join_fail())
    });
    c.bench_function("notification packet", |b| {
        b.iter(|| packets::notification("hello"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
