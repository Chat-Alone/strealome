use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    controller::{jwt::Jwt, Response},
    service::room, // <--- More specific import
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateRoomNameReq {
    name: String,
}

pub async fn update_room_name(
    Path(room_id): Path<String>,
    State(_state): State<AppState>,
    jwt: Jwt,
    Json(payload): Json<UpdateRoomNameReq>,
) -> impl IntoResponse {
    let room = match room::get_room_by_link(&room_id) { // <--- Direct call
        Ok(room) => room,
        Err(e) => return Response::error(&e.to_string()).into_response(),
    };

    if room.host_id() != jwt.sub {
        return (StatusCode::FORBIDDEN, "Only the host can change the room name").into_response();
    }

    match room::change_room_name(&room_id, payload.name).await { // <--- Direct call
        Ok(_) => Response::success::<()>(None).into_response(),
        Err(e) => Response::error(&e.to_string()).into_response(),
    }
}
