use criterion::{black_box, criterion_group, criterion_main, Criterion};
use turbo_zk_benchmark::udp_ping_pong::udp_ping_pong;

fn udp_ping_pong_benchmark(c: &mut Criterion) {
    c.bench_function("udp_ping_pong", |b| b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| udp_ping_pong()));
}

criterion_group!(benches, criterion_benchmark, udp_ping_pong_benchmark);
criterion_main!(benches); 