use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::service::room::Room;

#[derive(Serialize)]
pub struct RoomResp {
    name:       String,
    hosting:    bool,
    share_link: String,
    created_at: DateTime<Utc>,
}

impl RoomResp {
    pub fn from(room: Room, host_id: i32) -> Self {
        Self {
            name:       room.share_link(),
            share_link: room.share_link(),
            created_at: room.created_at(),
            hosting:    room.host_id() == host_id,
        }
    }
}