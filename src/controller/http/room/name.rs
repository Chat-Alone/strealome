use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    controller::{jwt::Jwt, Response},
    service::{room, user},
    model::chat::Event as ChatEvent,
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateRoomNameReq {
    name: String,
}

pub async fn update_room_name(
    jwt: Jwt,
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(payload): Json<UpdateRoomNameReq>,
) -> Response {
    let room = match state.rooms.get_room_by_link(&room_id) {
        Ok(room) => room,
        Err(e) => return e.into(),
    };

    if room.host_id() != jwt.sub {
        return Response::fail(StatusCode::FORBIDDEN, Some("Only the host can change the room name"));
    }

    match state.rooms.change_room_name(&room_id, payload.name.clone()).await {
        Ok(_) => {
            if let Ok(user) = user::get_user_by_id(state.repository.clone(), jwt.sub).await {
                let event = ChatEvent::room_name_updated(
                    payload.name.clone(),
                    jwt.sub,
                    user.name
                );
                
                if let Err(e) = room.sync_event(jwt.sub, event).await {
                    eprintln!("Failed to broadcast room name update event: {}", e);
                }
            }
            
            Response::success::<()>(None)
        },
        Err(e) => e.into(),
    }
}
