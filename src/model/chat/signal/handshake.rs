use serde::{Deserialize, Serialize};
use super::{Direction, DirectedPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandShake {
    pub id: i32,
    pub sn: i32,
}

impl DirectedPayload for HandShake {
    fn dir(&self) -> Direction {
        Direction::ToClient
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandShakeACK {
    pub id: i32,
    pub sn: i32,
}

impl DirectedPayload for HandShakeACK {
    fn dir(&self) -> Direction {
        Direction::ToServer
    }
}