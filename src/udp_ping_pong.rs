use std::time::Instant;
use tokio::net::UdpSocket;
use anyhow::Result;

pub async fn udp_ping_pong(iterations: u64, msg_size: usize) -> Result<(std::time::Duration, usize)> {
    let addr1 = "127.0.0.1:8888";
    let addr2 = "127.0.0.1:9999";

    let socket1 = UdpSocket::bind(addr1).await.unwrap();
    let socket2 = UdpSocket::bind(addr2).await.unwrap();

    let msg = vec![0; msg_size];

    let handle = tokio::spawn(async move {
        let mut buf = vec![0; msg_size];
        for _ in 0..iterations {
            let (len, _) = socket2.recv_from(&mut buf).await.unwrap();
            socket2.send_to(&buf[..len], addr1).await.unwrap();
        }
    });

    let start = Instant::now();

    for _ in 0..iterations {
        socket1.send_to(&msg, addr2).await.unwrap();
        let mut buf = vec![0; msg_size];
        socket1.recv_from(&mut buf).await.unwrap();
    }

    handle.await.unwrap();

    let elapsed = start.elapsed();
    let total_bytes = iterations * msg_size as u64 * 2;

    Ok((elapsed, total_bytes as usize))
} 