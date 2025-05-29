mod handshake;
mod ping;
mod event;

pub use handshake::{HandShake, HandShakeACK};
pub use ping::{Ping, Pong};
pub use event::{Event, Message};

use serde::{Deserialize, Serialize};
use super::{Author};

use std::sync::atomic::AtomicI32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    id:         i32,
    #[serde(rename = "a")]
    author_id:  Author,
    #[serde(flatten)]
    payload:    Payload,
}

impl Signal {
    pub fn new(id: i32, author_id: Author, payload: Payload) -> Self {
        Self {
            id,
            author_id,
            payload,
        }
    }
    
    pub fn event(id: i32, author_id: Author, event: Event) -> Self {
        Self::new(id, author_id, Payload::Event(event))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "p")]
pub enum Payload {
    #[serde(rename = "0")]
    HandShake(HandShake),
    #[serde(rename = "1")]
    HandShakeACK(HandShakeACK),
    #[serde(rename = "2")]
    Ping(Ping),
    #[serde(rename = "3")]
    Pong(Pong),
    #[serde(rename = "4")]
    Event(Event),
}

#[test]
fn test() {
    let signal = Signal {
        id: 1,
        author_id: Author::User(1),
        payload: Payload::HandShake(HandShake {
            id: 1,
            sn: 1,
        }),
    };
    
    println!("{}", serde_json::to_string(&signal).unwrap());
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    ToServer,
    ToClient,
}
impl Serialize for Direction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_u8(*self as u8)
    }
}
impl<'de> Deserialize<'de> for Direction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let u8 = u8::deserialize(deserializer)?;
        match u8 {
            0 => Ok(Direction::ToServer),
            1 => Ok(Direction::ToClient),
            _ => Err(serde::de::Error::custom("Invalid direction")),
        }
    }
}

pub trait DirectedPayload: Serialize + Deserialize<'static> {
    fn dir(&self) -> Direction;
}