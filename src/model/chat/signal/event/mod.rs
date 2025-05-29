mod join;
mod leave;
mod message;

pub use join::Join;
pub use leave::Leave;
pub use message::Message;

use serde::{ Deserialize, Serialize };
use super::{ DirectedPayload, Direction };

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "ev", content = "d")]
pub enum Event {
    Join(Join),
    Leave(Leave),
    Chat(Message),
}

impl Event {
    pub fn join(user_id: i32, capacity: usize) -> Self {
        Self::Join(Join { user_id, capacity })
    }

    pub fn leave(user_id: i32, capacity: usize) -> Self {
        Self::Leave(Leave { user_id, capacity })
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