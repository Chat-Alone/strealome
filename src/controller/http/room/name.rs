use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    controller::{jwt::Jwt, Response},
    service::room,
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateRoomNameReq {
    name: String,
}

pub async fn update_room_name(
    jwt: Jwt,
    Path(room_id): Path<String>,
    Json(payload): Json<UpdateRoomNameReq>,
    State(state): State<AppState>,
) -> Response {
    let room = match state.rooms.get_room_by_link(&room_id) {
        Ok(room) => room,
        Err(e) => return e.into(),
    };

    if room.host_id() != jwt.sub {
        return Response::fail(StatusCode::FORBIDDEN, Some("Only the host can change the room name"));
    }

    match state.rooms.change_room_name(&room_id, payload.name).await {
        Ok(_) => Response::success::<()>(None),
        Err(e) => e.into(),
    }
}
