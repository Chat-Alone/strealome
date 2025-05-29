use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use super::{AppState, Response, Jwt, RoomResp};
use crate::service::{room, user};

#[derive(Deserialize, Debug)]
struct PostRequest {
    name: String,
}

#[derive(Serialize, Debug)]
struct PostResponse {
    room: RoomResp,
}

async fn post(
    jwt: Jwt, State(state): State<AppState>,
    Json(req): Json<PostRequest>
) -> Response {
    let repo = state.repository;
    let user = user::get_user_by_id(repo.clone(), jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();
    
    let PostRequest { name: room_name } = req;
    let room = RoomResp::from(
        room::create_host_by(user.id, room_name),
        user.id, repo
    ).await;
    
    match room {
        Err(e) => e.into(),
        Ok(room) => Response::success(Some(PostResponse { room })),
    }
    
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::post(post))
}