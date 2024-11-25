use tokio::net::UdpSocket;

pub async fn udp_ping_pong() {
    let addr1 = "127.0.0.1:8888";
    let addr2 = "127.0.0.1:9999";

    let socket1 = UdpSocket::bind(addr1).await.unwrap();
    let socket2 = UdpSocket::bind(addr2).await.unwrap();

    let msg = b"ping";

    let handle = tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let (len, _) = socket2.recv_from(&mut buf).await.unwrap();
            socket2.send_to(&buf[..len], addr1).await.unwrap();
        }
    });

    for _ in 0..1000 {
        socket1.send_to(msg, addr2).await.unwrap();
        let mut buf = [0; 1024];
        socket1.recv_from(&mut buf).await.unwrap();
    }

    handle.abort();
} 