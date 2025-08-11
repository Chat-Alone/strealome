use axum::{extract::{Path, State}, Json, http::StatusCode};

use crate::{controller::{jwt::Jwt, Response, error::Error}, service};

use super::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateRoomNameReq {
    name: String,
}

pub async fn update_room_name(
    State(_state): State<AppState>,
    jwt: Jwt,
    Path(room_id): Path<String>,
    Json(payload): Json<UpdateRoomNameReq>,
) -> Result<Json<Response>, Error> {
    let room = service::room::get_room_by_link(&room_id)?;
    if room.host_id() != jwt.sub {
        return Err(Error::Forbidden("Only the host can change the room name"));
    }

    service::room::change_room_name(&room_id, payload.name).await?;

    Ok(Json(Response::ok(None)))
}
