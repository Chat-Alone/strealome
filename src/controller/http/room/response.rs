use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::service::room::Room;

#[derive(Serialize, Debug)]
pub struct RoomResp {
    name:       String,
    hosting:    bool,
    host:       String,
    share_link: String,
    member_cnt: usize,
    created_at: DateTime<Utc>,
}

impl RoomResp {
    pub fn from(room: Room, host_id: i32) -> Self {
        Self {
            name:       room.name(),
            host:       room.host_name(),
            share_link: room.share_link(),
            created_at: room.created_at(),
            member_cnt: room.user_len(),
            hosting:    room.host_id() == host_id,
        }
    }
}