use axum::extract::State;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::service::{user, room};
use crate::service::room::Room;
use super::super::{Jwt, AppState, Response};

#[derive(Serialize)]
struct RoomResp {
    name: String,
    share_link: String,
    created_at: DateTime<Utc>
}

impl From<Room> for RoomResp {
    fn from(room: Room) -> Self {
        Self {
            name:       room.share_link(),
            share_link: room.share_link(),
            created_at: room.created_at(),
        }
    }
}

#[derive(Serialize)]
struct GetResponse {
    related_rooms: Vec<RoomResp>,
}

#[derive(Serialize)]
struct PostResponse {
    room: RoomResp,
}

// get all related rooms
async fn get(jwt: Jwt, State(state): State<AppState>) -> Response {
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();
    
    let related_rooms = room::rooms()
            .related_rooms(user.id)
            .into_iter().map(|r| r.into()).collect();
    
    Response::success(Some(GetResponse { related_rooms }))
}

// create a room
async fn post(jwt: Jwt, State(state): State<AppState>) -> Response {
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();
    
    let room = room::rooms().create(user.id).into();
    Response::success(Some(PostResponse { room }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::get(get))
}