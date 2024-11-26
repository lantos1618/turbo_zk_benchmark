use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub async fn webrtc_benchmark(
    iterations: u64,
    msg_size: usize,
) -> Result<(std::time::Duration, usize)> {
    // Initialize MediaEngine and Interceptors
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;

    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Configure the STUN server (use a public one for testing)
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            // urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            urls: vec!["stun:localhost:3478".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Signaling channels for SDP and ICE candidates
    let (sdp_tx1, mut sdp_rx1) = mpsc::channel::<String>(1);
    let (sdp_tx2, mut sdp_rx2) = mpsc::channel::<String>(1);

    let (ice_tx1, mut ice_rx1) = mpsc::channel::<String>(10);
    let (ice_tx2, mut ice_rx2) = mpsc::channel::<String>(10);

    // Create Peer Connections
    let pc1 = Arc::new(api.new_peer_connection(config.clone()).await?);
    let pc2 = Arc::new(api.new_peer_connection(config).await?);

    // Set up ICE candidate exchange for pc1
    {
        let ice_tx1 = ice_tx1.clone();
        pc1.on_ice_candidate(Box::new(move |candidate| {
            let ice_tx1 = ice_tx1.clone();
            Box::pin(async move {
                if let Some(c) = candidate {
                    let cand = serde_json::to_string(&c.to_json().unwrap()).unwrap();
                    ice_tx1.send(cand).await.unwrap();
                }
            })
        }));
    }

    // Set up ICE candidate exchange for pc2
    {
        let ice_tx2 = ice_tx2.clone();
        pc2.on_ice_candidate(Box::new(move |candidate| {
            let ice_tx2 = ice_tx2.clone();
            Box::pin(async move {
                if let Some(c) = candidate {
                    let cand = serde_json::to_string(&c.to_json().unwrap()).unwrap();
                    ice_tx2.send(cand).await.unwrap();
                }
            })
        }));
    }

    // Create data channel only on pc1
    let dc = pc1.create_data_channel("data", None).await?;

    // Set up data channel on pc2 when it's created by pc1
    let (dc2_tx, mut dc2_rx) = mpsc::channel(1);
    pc2.on_data_channel(Box::new(move |dc| {
        let dc2_tx = dc2_tx.clone();
        Box::pin(async move {
            dc2_tx.send(dc).await.unwrap();
        })
    }));

    // SDP Offer/Answer exchange
    // Create offer on pc1
    let offer = pc1.create_offer(None).await?;
    pc1.set_local_description(offer.clone()).await?;

    // Wait for ICE gathering to complete on pc1
    let mut pc1_ice_gathering_complete = pc1.gathering_complete_promise().await;
    pc1_ice_gathering_complete.recv().await;

    let offer_sdp = serde_json::to_string(&pc1.local_description().await.unwrap())?;
    sdp_tx1.send(offer_sdp).await?;

    // pc2 receives the offer
    let offer_sdp = sdp_rx1.recv().await.unwrap();
    let offer = serde_json::from_str::<RTCSessionDescription>(&offer_sdp)?;
    pc2.set_remote_description(offer).await?;

    // Create answer on pc2
    let answer = pc2.create_answer(None).await?;
    pc2.set_local_description(answer.clone()).await?;

    // Wait for ICE gathering to complete on pc2
    let mut pc2_ice_gathering_complete = pc2.gathering_complete_promise().await;
    pc2_ice_gathering_complete.recv().await;

    let answer_sdp = serde_json::to_string(&pc2.local_description().await.unwrap())?;
    sdp_tx2.send(answer_sdp).await?;

    // pc1 receives the answer
    let answer_sdp = sdp_rx2.recv().await.unwrap();
    let answer = serde_json::from_str::<RTCSessionDescription>(&answer_sdp)?;
    pc1.set_remote_description(answer).await?;

    // Start exchanging ICE candidates
    let pc1_clone = pc1.clone();
    let pc2_clone = pc2.clone();

    let ice_exchange = async move {
        loop {
            tokio::select! {
                Some(ice) = ice_rx1.recv() => {
                    let candidate = serde_json::from_str::<RTCIceCandidateInit>(&ice)?;
                    pc2_clone.add_ice_candidate(candidate).await?;
                },
                Some(ice) = ice_rx2.recv() => {
                    let candidate = serde_json::from_str::<RTCIceCandidateInit>(&ice)?;
                    pc1_clone.add_ice_candidate(candidate).await?;
                },
                else => break,
            }
        }
        Result::<(), anyhow::Error>::Ok(())
    };

    // Start ICE exchange
    tokio::spawn(async move {
        if let Err(e) = ice_exchange.await {
            eprintln!("ICE exchange error: {:?}", e);
        }
    });

    // Wait for the peer connections to reach connected state
    let pc1_connected = wait_for_peer_connection(pc1.clone()).await?;
    let pc2_connected = wait_for_peer_connection(pc2.clone()).await?;

    if !pc1_connected || !pc2_connected {
        return Err(anyhow::anyhow!("Peer connections did not reach connected state"));
    }

    // Wait for data channels to open
    let dc1_ready = wait_for_data_channel_open(dc.clone()).await?;
    let dc2 = dc2_rx.recv().await.unwrap();
    let dc2_ready = wait_for_data_channel_open(dc2.clone()).await?;

    if !dc1_ready || !dc2_ready {
        return Err(anyhow::anyhow!("Data channels did not open"));
    }

    // Data receiving on dc2
    let (data_rx_tx, mut data_rx_rx) = mpsc::channel::<usize>(iterations as usize);
    dc2.on_message(Box::new(move |msg: DataChannelMessage| {
        let len = msg.data.len();
        let data_rx_tx = data_rx_tx.clone();
        Box::pin(async move {
            data_rx_tx.send(len).await.unwrap();
        })
    }));

    // Start benchmarking
    let start = Instant::now();
    let message = Bytes::from(vec![0u8; msg_size]);

    for _ in 0..iterations {
        dc.send(&message).await?;
    }

    // Collect received data sizes
    let mut total_bytes = 0;
    for _ in 0..iterations {
        if let Some(len) = data_rx_rx.recv().await {
            total_bytes += len;
        }
    }

    let elapsed = start.elapsed();

    // Close peer connections
    pc1.close().await?;
    pc2.close().await?;

    Ok((elapsed, total_bytes))
}

// Helper function to wait for the peer connection to reach connected state
async fn wait_for_peer_connection(pc: Arc<webrtc::peer_connection::RTCPeerConnection>) -> Result<bool> {
    let (connected_tx, connected_rx) = tokio::sync::oneshot::channel();
    let connected_tx = Arc::new(std::sync::Mutex::new(Some(connected_tx)));
    pc.on_peer_connection_state_change(Box::new(move |state| {
        let connected_tx = connected_tx.clone();
        Box::pin(async move {
            let mut tx_guard = connected_tx.lock().unwrap();
            if let Some(tx) = tx_guard.take() {
                if state == RTCPeerConnectionState::Connected {
                    let _ = tx.send(true);
                } else if state == RTCPeerConnectionState::Failed {
                    let _ = tx.send(false);
                }
            }
        })
    }));
    Ok(connected_rx.await?)
}

// Helper function to wait for the data channel to open
async fn wait_for_data_channel_open(dc: Arc<webrtc::data_channel::RTCDataChannel>) -> Result<bool> {
    let (open_tx, open_rx) = tokio::sync::oneshot::channel();
    dc.on_open(Box::new(move || {
        let _ = open_tx.send(true);
        Box::pin(async {})
    }));
    Ok(open_rx.await?)
}
