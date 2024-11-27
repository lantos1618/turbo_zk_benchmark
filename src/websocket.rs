use std::time::Instant;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::task;

pub async fn websocket_benchmark(iterations: usize, msg_size: usize, print_interval: usize) -> Result<(std::time::Duration, usize), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await?;
    println!("\nWebSocket server listening on: {}", addr);

    let msg = vec![0; msg_size];

    let (stream, _) = listener.accept().await?;
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    let start = Instant::now();
    for i in 0..iterations {
        write.send(Message::Binary(msg.clone())).await?;
        let _ = read.next().await.ok_or("Failed to receive message")??;
        
        if i % print_interval == 0 {
            let elapsed = start.elapsed();
            let latency_ms = elapsed.as_nanos() as f64 / 1_000_000.0 / (i + 1) as f64;
            let throughput_mbps = msg_size as f64 * (i + 1) as f64 / elapsed.as_secs_f64() / 1_000_000.0;
            println!(
                "Iteration {}: Latency: {:.2} ms/iter, Throughput: {:.2} MB/s",
                i, latency_ms, throughput_mbps
            );
        }
    }
    let elapsed = start.elapsed();
    let total_bytes = msg_size * iterations * 2; // Account for both send and receive

    Ok((elapsed, total_bytes))
}
