use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomNameUpdated {
    pub new_name: String,
    pub updater_id: i32,
    pub updater_name: String,
}
