use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use turbo_zk_benchmark::udp_ping_pong::udp_ping_pong;
use turbo_zk_benchmark::webrtc_benchmark::webrtc_benchmark;
use turbo_zk_benchmark::websocket::websocket_benchmark;
use turbo_zk_benchmark::zk_bellman::zk_bellman_benchmark;

fn udp_ping_pong_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("udp_ping_pong");

    let iterations = 1000;
    let msg_size = 1024;

    group.bench_function("udp_ping_pong", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            if let Ok((elapsed, total_bytes)) = rt.block_on(udp_ping_pong(black_box(iterations), black_box(msg_size))) {
                let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / iterations as f64;
                let throughput_mbps = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
                println!("udp_ping_pong: Latency: {:.2} ms/iter, Throughput: {:.2} MB/s", latency_ms, throughput_mbps);
            } else {
                println!("Error occurred during UDP ping pong benchmark");
            }
        })
    });

    group.finish();
}


fn webrtc_benchmark_fn(c: &mut Criterion) {
    let mut group = c.benchmark_group("webrtc");

    let iterations = 1000;
    let msg_size = 1024;

    // Create a single Tokio runtime outside the loop
    let rt = Runtime::new().unwrap();
    group.bench_function("webrtc", |b| {
        b.iter(|| {
            rt.block_on(async {
                match webrtc_benchmark(black_box(iterations), black_box(msg_size)).await {
                    Ok((elapsed, total_bytes)) => {
                        let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / iterations as f64;
                        let throughput_mbps = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
                    println!(
                        "Latency: {:.2} ms/iter, Throughput: {:.2} MB/s",
                        latency_ms, throughput_mbps
                    );
                }
                Err(e) => {
                    println!("Error occurred during WebRTC benchmark: {:?}", e);
                }
            }
        })
        });
    });

    group.finish();
}

fn zk_bellman_benchmark_fn(c: &mut Criterion) {
    let mut group = c.benchmark_group("zk_bellman");
    group.measurement_time(std::time::Duration::from_secs(60));

    let iterations = 100;
    // let payload_size = 1024;
    let payload_size = 8192;

    group.bench_function("zk_bellman", |b| {
        b.iter(|| {
            if let Ok((elapsed, total_bytes)) = zk_bellman_benchmark(black_box(payload_size), black_box(iterations)) {
                let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / iterations as f64;
                let throughput_mbps = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
                println!("zk_bellman: Latency: {:.2} ms/iter, Throughput: {:.2} MB/s", latency_ms, throughput_mbps);
            } else {
                println!("Error occurred during ZK Bellman benchmark");
            }
        })
    });

    group.finish();
}

fn websocket_benchmark_fn(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket");

    let iterations = 1000;
    let msg_size = 1024;

    let rt = Runtime::new().unwrap();
    group.bench_function("websocket", |b| {
        b.iter(|| {
            rt.block_on(async {
                match websocket_benchmark(black_box(iterations), black_box(msg_size), black_box(100)).await {
                    Ok((elapsed, total_bytes)) => {
                        let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / iterations as f64;
                        let throughput_mbps = total_bytes as f64 / elapsed.as_secs_f64() / 1_000_000.0;
                        println!(
                            "WebSocket: Latency: {:.2} ms/iter, Throughput: {:.2} MB/s",
                            latency_ms, throughput_mbps
                        );
                    }
                    Err(e) => {
                        println!("Error occurred during WebSocket benchmark: {:?}", e);
                    }
                }
            })
        });
    });

    group.finish();
}

criterion_group!(benches,websocket_benchmark_fn, udp_ping_pong_benchmark, webrtc_benchmark_fn, zk_bellman_benchmark_fn, );
criterion_main!(benches); 
