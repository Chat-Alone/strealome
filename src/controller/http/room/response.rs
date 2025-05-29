use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::repository::Repository;
use crate::service::room::{Room, RoomError};

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
    pub async fn from(room: Room, user_id: i32, repo: Arc<dyn Repository>) -> Result<Self, RoomError> {
        Ok(Self {
            name:       room.name(),
            host:       room.host_name(repo).await?,
            share_link: room.share_link(),
            created_at: room.created_at(),
            member_cnt: room.user_len(),
            hosting:    room.host_id() == user_id,
        })
    }
}