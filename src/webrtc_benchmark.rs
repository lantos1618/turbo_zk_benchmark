use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub async fn webrtc_benchmark(
    iterations: u64,
    msg_size: usize,
) -> Result<(std::time::Duration, usize)> {
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;

    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let pc1 = Arc::new(api.new_peer_connection(config.clone()).await?);
    let pc2 = Arc::new(api.new_peer_connection(config).await?);

    let (offer_tx, offer_rx) = mpsc::channel::<String>(1);
    let (answer_tx, answer_rx) = mpsc::channel::<String>(1);

    let pc1_offer_tx = offer_tx.clone();
    let mut pc1_answer_rx = answer_rx;
    let pc1_clone = pc1.clone();

    let mut pc2_offer_rx = offer_rx;
    let pc2_answer_tx = answer_tx.clone();
    let pc2_clone = pc2.clone();

    let mut total_bytes = 0;
    let message = Bytes::from(vec![0u8; msg_size]);

    let start = Instant::now();

    let dc1_fut = async move {
        let dc1 = pc1_clone
            .create_data_channel("test", None)
            .await
            .expect("Failed to create data channel");

        let offer = pc1_clone.create_offer(None).await.unwrap();
        let offer_sdp = serde_json::to_string(&offer).unwrap();
        pc1_offer_tx.send(offer_sdp).await.unwrap();

        let answer_sdp = pc1_answer_rx.recv().await.unwrap();
        let answer = serde_json::from_str::<RTCSessionDescription>(&answer_sdp).unwrap();
        pc1_clone.set_remote_description(answer).await.unwrap();

        let dc1_clone = dc1.clone();
        dc1.on_open(Box::new(move || {
            println!("Data channel 1 opened");
            Box::pin(async move {
                for _ in 0..iterations {
                    dc1_clone.send(&message).await.unwrap();
                    total_bytes += msg_size;
                }
            })
        }));
    };

    let dc2_fut = async move {
        let offer_sdp = pc2_offer_rx.recv().await.unwrap();
        let offer = serde_json::from_str::<RTCSessionDescription>(&offer_sdp).unwrap();
        pc2_clone.set_remote_description(offer).await.unwrap();

        let answer = pc2_clone.create_answer(None).await.unwrap();
        let answer_sdp = serde_json::to_string(&answer).unwrap();
        pc2_answer_tx.send(answer_sdp).await.unwrap();

        let dc2 = pc2_clone.create_data_channel("test", None).await.unwrap();

        dc2.on_message(Box::new(move |msg: DataChannelMessage| {
            println!("Received message of length {}", msg.data.len());
            Box::pin(async {})
        }));
    };

    tokio::join!(dc1_fut, dc2_fut);

    let elapsed = start.elapsed();

    Ok((elapsed, total_bytes))
}
