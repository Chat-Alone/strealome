use axum::{Router, routing, Json};
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::extract::State;
use axum::http::{header, StatusCode};
use serde::{Serialize};
use super::{Jwt, AppState};
use crate::model::whip::SessionDescModel;
use crate::service::{user, webrtc};
use crate::unwrap;

async fn post(
    // jwt: Jwt, State(s): State<AppState>,
    offer: SessionDescModel
) -> AxumResponse {
    // let user_id = jwt.sub;
    // unwrap!(user::get_user_by_id(s.repository, user_id).await);
    // 
    
    println!("offer: {:?}", offer);
    let res = webrtc::handle_whip(114514, offer.sdp()).await;
    if let Err(_e) = res {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
    let (pc, ans) = res.unwrap();
    
    
    let mut resp = (StatusCode::CREATED, ans.sdp).into_response();
    resp.headers_mut().insert("Accept-Patch", "application/trickle-ice-sdpfrag".parse().unwrap());
    resp.headers_mut().insert(header::CONTENT_TYPE, "application/sdp".parse().unwrap());
    resp.headers_mut().insert(header::LOCATION, "/s/res".parse().unwrap());
    resp.headers_mut().insert(header::LINK, "stun:stun.l.google.com:19302".parse().unwrap());
    
    resp
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::post(post))
}