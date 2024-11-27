use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use url::Url;
use std::time::{Duration, Instant};
const MESSAGE_SIZE_BYTES: usize = 32; // Size of the ping message in bytes

async fn ping_pong_websocket() {
    let handles = vec![
        tokio::spawn(async move {
            // server thread
            let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
            println!("WebSocket server listening on ws://127.0.0.1:9001");

            while let Ok((stream, _)) = listener.accept().await {
                let ws_stream = accept_async(stream).await.unwrap();
                let (mut write, mut read) = ws_stream.split();

                loop {
                    // Server sends a ping
                    let start = Instant::now();
                    write.send(Message::Ping(vec![0; MESSAGE_SIZE_BYTES])).await.unwrap();
                    
                    // Wait for a pong from the client
                    if let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Pong(_)) => {
                                // send a ping back to the client
                                write.send(Message::Ping(vec![0; MESSAGE_SIZE_BYTES])).await.unwrap();
                            }
                            Ok(_) => {}
                            Err(e) => {
                                println!("Server error: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                // Close the connection properly
                write.close().await.unwrap();
            }
        }),
        tokio::spawn(async move {
            // client thread
            let url = Url::parse("ws://127.0.0.1:9001").unwrap();
            let (ws_stream, _) = connect_async(url.as_str()).await.unwrap();
            let (mut write, mut read) = ws_stream.split();

            let mut message_count = 0;
            let start_time = Instant::now();

            loop {
                // Wait for a ping from the server
                if let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Ping(_)) => {
                            // Respond with a pong
                            write.send(Message::Pong(vec![0; MESSAGE_SIZE_BYTES])).await.unwrap();
                            message_count += 1;
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let throughput = (message_count * MESSAGE_SIZE_BYTES * 8) as f64 / (elapsed * 1_000_000.0);
                            println!("Client sent pong, throughput: {:.2} Mb/s", throughput);

                            // Send a ping back to the server
                            write.send(Message::Ping(vec![0; MESSAGE_SIZE_BYTES])).await.unwrap();
                        }
                        Ok(Message::Pong(_)) => {
                            // Server should not send pongs, ignore
                        }
                        Ok(_) => {}
                        Err(e) => {
                            println!("Client error: {:?}", e);
                            break;
                        }
                    }
                }
            }
            // Close the connection properly
            write.close().await.unwrap();
        })
    ];

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    ping_pong_websocket().await;

}
