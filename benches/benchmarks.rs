use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use turbo_zk_benchmark::udp_ping_pong::udp_ping_pong;

fn udp_ping_pong_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("udp_ping_pong");
    group.throughput(criterion::Throughput::Elements(1));

    let iterations = 1000;

    group.bench_function("udp_ping_pong", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| rt.block_on(udp_ping_pong(black_box(iterations))))
    });

    group.finish();
}

criterion_group!(benches, udp_ping_pong_benchmark);
criterion_main!(benches); 