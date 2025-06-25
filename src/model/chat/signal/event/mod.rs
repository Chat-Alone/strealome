mod join;
mod leave;
mod message;
mod transfer;

pub use join::Join;
pub use leave::Leave;
pub use message::Message;
pub use transfer::Transfer;

use serde::{ Deserialize, Serialize };
use super::{ DirectedPayload, Direction };

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "ev", content = "d")]
pub enum Event {
    #[serde(rename = "join")]
    Join(Join),
    #[serde(rename = "leave")]
    Leave(Leave),
    #[serde(rename = "transfer")]
    Transfer(Transfer),
    #[serde(rename = "chat")]
    Chat(Message),
}

impl Event {
    pub fn join(user_id: i32, capacity: usize) -> Self {
        Self::Join(Join { user_id, capacity })
    }

    pub fn leave(user_id: i32, capacity: usize) -> Self {
        Self::Leave(Leave { user_id, capacity })
    }
    
    pub fn chat(msg: Message) -> Self {
        Self::Chat(msg)
    }
    
    pub fn transfer(host_id: i32) -> Self {
        Self::Transfer(Transfer { host_id })
    }
    
}

impl DirectedPayload for Event {
    fn dir(&self) -> Direction {
        Direction::ToClient
    }
}