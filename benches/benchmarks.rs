use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use turbo_zk_benchmark::udp_ping_pong::udp_ping_pong;
use turbo_zk_benchmark::webrtc_benchmark::webrtc_benchmark;

fn udp_ping_pong_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("udp_ping_pong");

    let iterations = 10000;
    let msg_size = 1024;

    group.bench_function("udp_ping_pong", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            let (elapsed, total_bytes) = rt.block_on(udp_ping_pong(black_box(iterations), black_box(msg_size)));
            let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / iterations as f64;
            let throughput_mbps = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
            println!("Latency: {:.2} ms/iter, Throughput: {:.2} MB/s", latency_ms, throughput_mbps);
        })
    });

    group.finish();
}

fn webrtc_benchmark_fn(c: &mut Criterion) {
    let mut group = c.benchmark_group("webrtc");

    let iterations = 1000;
    let msg_size = 1024;

    group.bench_function("webrtc", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            let (elapsed, total_bytes) = rt.block_on(webrtc_benchmark(black_box(iterations), black_box(msg_size)));
            let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / iterations as f64;
            let throughput_mbps = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
            println!("Latency: {:.2} ms/iter, Throughput: {:.2} MB/s", latency_ms, throughput_mbps);
        })
    });

    group.finish();
}

criterion_group!(benches, udp_ping_pong_benchmark, webrtc_benchmark_fn);
criterion_main!(benches); 