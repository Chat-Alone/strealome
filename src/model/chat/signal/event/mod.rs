mod join;
mod leave;
mod chat;

pub use join::Join;
pub use leave::Leave;
pub use chat::Chat;

use serde::{ Deserialize, Serialize };
use super::{ DirectedPayload, Direction };

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "ev", content = "d")]
pub enum Event {
    Join(Join),
    Leave(Leave),
    Chat(Chat),
}

impl Event {
    pub fn join(user_id: i32, room: String) -> Self {
        todo!()
    }
    
    pub fn chat() -> Self {
        todo!()
    }
}

impl DirectedPayload for Event {
    fn dir(&self) -> Direction {
        match self {
            _ => Direction::ToClient
        }
    }
}