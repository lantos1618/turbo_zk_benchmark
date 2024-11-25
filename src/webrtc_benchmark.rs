use std::time::Instant;
use tokio::sync::mpsc;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use bytes::Bytes;


pub async fn webrtc_benchmark(iterations: u64, msg_size: usize) -> (std::time::Duration, usize) {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs().unwrap();

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let (offer_tx, mut offer_rx) = mpsc::channel(1);
    let (answer_tx, mut answer_rx) = mpsc::channel(1);

    let pc1 = api.new_peer_connection(config).await.unwrap();
    let pc2 = api.new_peer_connection(config).await.unwrap();

    let mut total_bytes = 0;

    let start = Instant::now();

    for _ in 0..iterations {
        let (data_channel_tx, mut data_channel_rx) = mpsc::channel(1);

        let dc1 = pc1
            .create_data_channel("benchmark", None)
            .await
            .unwrap();

        let dc1_clone = dc1.clone();
        pc2.on_data_channel(Box::new(move |dc| {
            Box::pin(async move {
                while let Ok(msg) = dc.recv().await {
                    total_bytes += msg.len();
                    dc1_clone.send(&Bytes::from(msg)).await.unwrap();
                }
                data_channel_tx.send(()).await.unwrap();
            })
        }));

        let offer = pc1.create_offer(None).await.unwrap();
        let mut gather_complete = pc1.gathering_complete_promise().await;
        pc1.set_local_description(offer).await.unwrap();
        let _ = gather_complete.recv().await;

        offer_tx.send(pc1.local_description().await.unwrap()).await.unwrap();
        let offer = offer_rx.recv().await.unwrap();

        pc2.set_remote_description(offer).await.unwrap();
        let answer = pc2.create_answer(None).await.unwrap();
        gather_complete = pc2.gathering_complete_promise().await;
        pc2.set_local_description(answer).await.unwrap();
        let _ = gather_complete.recv().await;

        answer_tx.send(pc2.local_description().await.unwrap()).await.unwrap();
        let answer = answer_rx.recv().await.unwrap();

        pc1.set_remote_description(answer).await.unwrap();

        let message = Bytes::from(vec![0u8; msg_size]);
        for _ in 0..iterations {
            dc1.send(&message).await.unwrap();
            total_bytes += msg_size;
        }

        let _ = data_channel_rx.recv().await;
    }

    let elapsed = start.elapsed();

    (elapsed, total_bytes)
} 