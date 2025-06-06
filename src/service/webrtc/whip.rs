use std::sync::Arc;
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::rtp_transceiver::rtp_codec::RTPCodecType;
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use super::Error;

pub async fn handle_whip(
    user_id: i32, offer: RTCSessionDescription
) -> Result<(Arc<RTCPeerConnection>, RTCSessionDescription), Error> {
    let mut m = MediaEngine::default();

    m.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    peer_connection
        .add_transceiver_from_kind(
            RTPCodecType::Video,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: Vec::new(),
            }),
        )
        .await?;
    
    peer_connection.on_track(Box::new(move |track, _, _| {
        println!("{:?}", track);
        Box::pin(async {})
    }));
    
    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            log::info!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Connected {
                log::info!("Ctrl+C the remote client to stop the demo");
            }
            Box::pin(async {})
        },
    ));
    
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        log::info!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            println!("Peer Connection has gone to failed exiting: Done forwarding");
        }

        Box::pin(async {})
    }));
    
    peer_connection.set_remote_description(offer).await?;
    let answer = peer_connection.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;
    
    
    match peer_connection.local_description().await {
        Some(desc) => {
            Ok((peer_connection, desc))
        }
        None => Err(Error::custom("我测".to_string())),
    }
    

}

fn answer_sdp(sdp: RTCSessionDescription) -> Result<RTCSessionDescription, Error> {
    todo!()
}