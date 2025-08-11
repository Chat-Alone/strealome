use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::{
    controller::{
        error::Error,
        jwt::{Jwt, Role},
        Response,
    },
    service,
};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateRoomNameReq {
    name: String,
}

pub async fn update_room_name(
    State(state): State<AppState>,
    jwt: Jwt,
    Path(room_id): Path<String>,
    Json(payload): Json<UpdateRoomNameReq>,
) -> Result<Json<Response>, Error> {
    jwt.check_role(Role::User)?;

    let room = service::room::get_room_by_link(&room_id)?;
    if room.host_id() != jwt.user_id {
        return Err(Error::Forbidden("Only the host can change the room name"));
    }

    service::room::change_room_name(&room_id, payload.name).await?;

    Ok(Json(Response::ok(None)))
}
