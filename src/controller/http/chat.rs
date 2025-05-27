use axum::extract::{State, Json};
use serde::Deserialize;
use crate::model::ChatMessageContent;
use super::{AppState, Jwt, Response};
use crate::service::{room, user};

#[derive(Debug, Deserialize)]
struct PostRequest {
    room:       String,
    content:    String,
}

async fn post(jwt: Jwt, State(state): State<AppState>, Json(req): Json<PostRequest>) -> Response {
    let user = user::get_user_by_id(state.repository, jwt.sub).await;
    if let Err(e) = user { return e.into() }
    let user = user.unwrap();
    
    let room = room::rooms().get_room_by_link(&req.room);
    if let Err(e) = room { return e.into() }
    
    match room.unwrap().sync_message(user.id, ChatMessageContent::Text(req.content)).await {
        Err(e) => e.into(),
        Ok(_) => Response::success::<()>(None)
    }
}

pub async fn route(path: &str) -> axum::Router<AppState> {
    axum::Router::new().route(path, axum::routing::post(post))
}