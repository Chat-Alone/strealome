use serde::{Deserialize, Serialize};
use super::{Direction, DirectedPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub id: i32,
    pub sn: i32,
}

impl DirectedPayload for Ping {
    fn dir(&self) -> Direction {
        Direction::ToClient
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    pub id: i32,
    pub sn: i32,
}

impl DirectedPayload for Pong {
    fn dir(&self) -> Direction {
        Direction::ToServer
    }
}