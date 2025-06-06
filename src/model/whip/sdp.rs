use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use super::Response;
const APPLICATION_SDP: &[u8] = "application/sdp".as_bytes();

#[derive(Debug)]
pub struct SessionDescModel(RTCSessionDescription);

impl SessionDescModel {
    pub fn sdp(self) -> RTCSessionDescription {
        self.0
    }
}

impl<S: Send + Sync> FromRequest<S> for SessionDescModel {
    type Rejection = Response;

    async fn from_request(req: Request, _s: &S) -> Result<Self, Self::Rejection> {
        if !req.headers().get("content-type")
            .map(|h| {
                println!("{:?}, {:?}", h.as_bytes(), APPLICATION_SDP);
                h.as_bytes() == APPLICATION_SDP
            }).unwrap_or(false)
        {
            return Err(Response::code(StatusCode::UNSUPPORTED_MEDIA_TYPE))
        };
        
        if let Ok(sdp) = Bytes::from_request(req, _s).await {
            if let Ok(sdp) = String::from_utf8(sdp.into_iter().collect()) {
                if let Ok(sdp) = RTCSessionDescription::offer(sdp) {
                    return Ok(SessionDescModel(sdp))
                }
            }
        }
        
        Err(Response::code(StatusCode::BAD_REQUEST))
    }
}