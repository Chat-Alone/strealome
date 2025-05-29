use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Leave {
    pub user:       String,
    pub capacity:   u32,
}