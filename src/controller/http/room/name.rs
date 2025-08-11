use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{
    controller::{
        jwt::Jwt,
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
    State(_state): State<AppState>,
    jwt: Jwt,
    Path(room_id): Path<String>,
    Json(payload): Json<UpdateRoomNameReq>,
) -> impl IntoResponse {
    match service::room::get_room_by_link(&room_id) {
        Ok(room) => {
            if room.host_id() != jwt.sub {
                return (StatusCode::FORBIDDEN, "Only the host can change the room name").into_response();
            }

            match service::room::change_room_name(&room_id, payload.name).await {
                Ok(_) => Response::success::<()>(None).into_response(),
                Err(e) => Response::from(e).into_response(),
            }
        }
        Err(e) => Response::from(e).into_response(),
    }
}