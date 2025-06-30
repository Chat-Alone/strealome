
use axum::{routing, Router};
use axum::extract::{Query, State};
use serde::Deserialize;
use super::{Jwt, AppState, Response, RoomResp};

#[derive(Deserialize, Debug)]
struct GetRequest {
    room: String
}

async fn get(jwt: Jwt, State(state): State<AppState>, Query(req): Query<GetRequest>) -> Response {
    let room = state.rooms.get_room_by_link(req.room.as_str());
    if let Err(e) = room { return e.into() }
    let room = room.unwrap();
    
    if let Err(e) = room.contains_user(jwt.sub) { return e.into() }
    let resp = RoomResp::from(room, jwt.sub, state.repository).await;
    match resp {
        Ok(resp) => Response::success(Some(resp)),
        Err(e) => e.into()
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, routing::get(get))
}